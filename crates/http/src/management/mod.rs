/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

pub mod crypto;
pub mod dkim;
pub mod dns;
pub mod log;
pub mod principal;
pub mod queue;
pub mod reload;
pub mod report;
pub mod settings;
pub mod spam;
pub mod stores;
pub mod troubleshoot;

// SPDX-SnippetBegin
// SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
// SPDX-License-Identifier: LicenseRef-SEL
#[cfg(feature = "enterprise")]
pub mod enterprise;

#[cfg(feature = "enterprise")]
use enterprise::telemetry::TelemetryApi;
// SPDX-SnippetEnd

use crate::auth::oauth::auth::OAuthApiHandler;
use common::{Server, auth::AccessToken};
use crypto::CryptoHandler;
use directory::{Permission, backend::internal::manage};
use dkim::DkimManagement;
use dns::DnsManagement;
use http_proto::{request::fetch_body, *};
use hyper::{Method, StatusCode, header};
use log::LogManagement;
use mail_parser::DateTime;
use principal::PrincipalManager;
use queue::QueueManagement;
use reload::ManageReload;
use report::ManageReports;
use serde::Serialize;
use settings::ManageSettings;
use spam::ManageSpamHandler;
use std::future::Future;
use std::{str::FromStr, sync::Arc};
use store::write::now;
use stores::ManageStore;
use troubleshoot::TroubleshootApi;

/// Convert a `trc::Error` to an `HttpResponse` using standard HTTP problem JSON.
fn trc_error_to_http_response(err: &trc::Error) -> HttpResponse {
    let details_or_reason = err
        .value(trc::Key::Details)
        .or_else(|| err.value(trc::Key::Reason))
        .and_then(|v| v.as_str());

    let (status, title, detail) = match err.as_ref() {
        trc::EventType::Limit(cause) => match cause {
            trc::LimitEvent::TooManyRequests => (
                429u16,
                "Too Many Requests",
                "Your request has been rate limited. Please try again in a few seconds.",
            ),
            _ => (400, "Bad Request", "Request limit exceeded."),
        },
        trc::EventType::Auth(cause) => match cause {
            trc::AuthEvent::TooManyAttempts => (
                429,
                "Too Many Authentication Attempts",
                "Your request has been rate limited. Please try again in a few minutes.",
            ),
            _ => (401, "Unauthorized", "You have to authenticate first."),
        },
        trc::EventType::Security(cause) => match cause {
            trc::SecurityEvent::Unauthorized => (
                403,
                "Forbidden",
                "You do not have enough permissions to access this resource.",
            ),
            _ => (
                429,
                "Too Many Authentication Attempts",
                "Your request has been rate limited. Please try again in a few minutes.",
            ),
        },
        trc::EventType::Resource(cause) => match cause {
            trc::ResourceEvent::NotFound => (
                404,
                "Not Found",
                "The requested resource does not exist on this server.",
            ),
            trc::ResourceEvent::BadParameters => (
                400,
                "Invalid parameters",
                details_or_reason.unwrap_or("One or multiple parameters could not be parsed."),
            ),
            _ => (
                500,
                "Internal Server Error",
                "There was a problem while processing your request.",
            ),
        },
        _ => (
            500,
            "Internal Server Error",
            "There was a problem while processing your request.",
        ),
    };

    let body = serde_json::json!({
        "type": "about:blank",
        "status": status,
        "title": title,
        "detail": detail,
    });

    HttpResponse::new(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .with_content_type("application/problem+json")
        .with_text_body(serde_json::to_string(&body).unwrap_or_default())
}

#[derive(Serialize)]
#[serde(tag = "error")]
#[serde(rename_all = "camelCase")]
pub enum ManagementApiError<'x> {
    FieldAlreadyExists {
        field: &'x str,
        value: &'x str,
    },
    FieldMissing {
        field: &'x str,
    },
    NotFound {
        item: &'x str,
    },
    Unsupported {
        details: &'x str,
    },
    AssertFailed,
    Other {
        details: &'x str,
        reason: Option<&'x str>,
    },
}

pub trait ManagementApi: Sync + Send {
    fn handle_api_manage_request(
        &self,
        req: &mut HttpRequest,
        access_token: Arc<AccessToken>,
        session: &HttpSessionData,
    ) -> impl Future<Output = trc::Result<HttpResponse>> + Send;
}

impl ManagementApi for Server {
    #[allow(unused_variables)]
    async fn handle_api_manage_request(
        &self,
        req: &mut HttpRequest,
        access_token: Arc<AccessToken>,
        session: &HttpSessionData,
    ) -> trc::Result<HttpResponse> {
        let body = fetch_body(req, 1024 * 1024, session.session_id).await;
        let path = req.uri().path().split('/').skip(2).collect::<Vec<_>>();

        match path.first().copied().unwrap_or_default() {
            "queue" => self.handle_manage_queue(req, path, &access_token).await,
            "settings" => {
                self.handle_manage_settings(req, path, body, &access_token)
                    .await
            }
            "reports" => self.handle_manage_reports(req, path, &access_token).await,
            "principal" => {
                self.handle_manage_principal(req, path, body, &access_token)
                    .await
            }
            "dns" => self.handle_manage_dns(req, path, &access_token).await,
            "store" => {
                self.handle_manage_store(req, path, body, session, &access_token)
                    .await
            }
            "spam-filter" => {
                self.handle_manage_spam(req, path, body, session, &access_token)
                    .await
            }
            "reload" => self.handle_manage_reload(req, path, &access_token).await,
            "dkim" => {
                self.handle_manage_dkim(req, path, body, &access_token)
                    .await
            }
            "update" => self.handle_manage_update(req, path, &access_token).await,
            "logs" if req.method() == Method::GET => {
                self.handle_view_logs(req, &access_token).await
            }
            "restart" if req.method() == Method::GET => {
                // Validate the access token
                access_token.assert_has_permission(Permission::Restart)?;

                Err(manage::unsupported("Restart is not yet supported"))
            }
            "oauth" => {
                // Validate the access token
                access_token.assert_has_permission(Permission::AuthenticateOauth)?;

                self.handle_oauth_api_request(access_token, body).await
            }
            "account" => match (path.get(1).copied().unwrap_or_default(), req.method()) {
                ("crypto", &Method::POST) => {
                    // Validate the access token
                    access_token.assert_has_permission(Permission::ManageEncryption)?;

                    self.handle_crypto_post(access_token, body).await
                }
                ("crypto", &Method::GET) => {
                    // Validate the access token
                    access_token.assert_has_permission(Permission::ManageEncryption)?;

                    self.handle_crypto_get(access_token).await
                }
                ("auth", &Method::GET) => {
                    // Validate the access token
                    access_token.assert_has_permission(Permission::ManagePasswords)?;

                    self.handle_account_auth_get(access_token).await
                }
                ("auth", &Method::POST) => {
                    // Validate the access token
                    access_token.assert_has_permission(Permission::ManagePasswords)?;

                    self.handle_account_auth_post(req, access_token, body).await
                }
                _ => Err(trc::ResourceEvent::NotFound.into_err()),
            },
            "troubleshoot" => {
                // Validate the access token
                access_token.assert_has_permission(Permission::Troubleshoot)?;

                self.handle_troubleshoot_api_request(req, path, &access_token, body)
                    .await
            }
            // SPDX-SnippetBegin
            // SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
            // SPDX-License-Identifier: LicenseRef-SEL
            #[cfg(feature = "enterprise")]
            "telemetry" => {
                // WARNING: TAMPERING WITH THIS FUNCTION IS STRICTLY PROHIBITED
                // Any attempt to modify, bypass, or disable this license validation mechanism
                // constitutes a severe violation of the Stalwart Enterprise License Agreement.
                // Such actions may result in immediate termination of your license, legal action,
                // and substantial financial penalties. Stalwart Labs LLC actively monitors for
                // unauthorized modifications and will pursue all available legal remedies against
                // violators to the fullest extent of the law, including but not limited to claims
                // for copyright infringement, breach of contract, and fraud.

                if self.core.is_enterprise_edition() {
                    self.handle_telemetry_api_request(req, path, &access_token)
                        .await
                } else {
                    Err(manage::enterprise())
                }
            }
            // SPDX-SnippetEnd
            _ => Err(trc::ResourceEvent::NotFound.into_err()),
        }
    }
}

pub(super) struct FutureTimestamp(u64);
pub(super) struct Timestamp(u64);

impl FromStr for Timestamp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(dt) = DateTime::parse_rfc3339(s) {
            Ok(Timestamp(dt.to_timestamp() as u64))
        } else {
            Err(())
        }
    }
}

impl FromStr for FutureTimestamp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(dt) = DateTime::parse_rfc3339(s) {
            let instant = dt.to_timestamp() as u64;
            if instant >= now() {
                return Ok(FutureTimestamp(instant));
            }
        }

        Err(())
    }
}

impl FutureTimestamp {
    pub fn into_inner(self) -> u64 {
        self.0
    }
}

impl Timestamp {
    pub fn into_inner(self) -> u64 {
        self.0
    }
}

pub trait ToManageHttpResponse {
    fn into_http_response(self) -> HttpResponse;
}

impl ToManageHttpResponse for &trc::Error {
    fn into_http_response(self) -> HttpResponse {
        match self.as_ref() {
            trc::EventType::Manage(cause) => {
                match cause {
                    trc::ManageEvent::MissingParameter => ManagementApiError::FieldMissing {
                        field: self.value_as_str(trc::Key::Key).unwrap_or_default(),
                    },
                    trc::ManageEvent::AlreadyExists => ManagementApiError::FieldAlreadyExists {
                        field: self.value_as_str(trc::Key::Key).unwrap_or_default(),
                        value: self.value_as_str(trc::Key::Value).unwrap_or_default(),
                    },
                    trc::ManageEvent::NotFound => ManagementApiError::NotFound {
                        item: self.value_as_str(trc::Key::Key).unwrap_or_default(),
                    },
                    trc::ManageEvent::NotSupported => ManagementApiError::Unsupported {
                        details: self
                            .value(trc::Key::Details)
                            .or_else(|| self.value(trc::Key::Reason))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Requested action is unsupported"),
                    },
                    trc::ManageEvent::AssertFailed => ManagementApiError::AssertFailed,
                    trc::ManageEvent::Error => ManagementApiError::Other {
                        reason: self.value_as_str(trc::Key::Reason),
                        details: self
                            .value_as_str(trc::Key::Details)
                            .unwrap_or("Unknown error"),
                    },
                }
            }
            .into_http_response(),
            trc::EventType::Auth(
                trc::AuthEvent::Failed | trc::AuthEvent::Error | trc::AuthEvent::TokenExpired,
            ) => HttpResponse::unauthorized(true),
            _ => trc_error_to_http_response(self),
        }
    }
}

pub trait UnauthorizedResponse {
    fn unauthorized(include_realms: bool) -> Self;
}

impl UnauthorizedResponse for HttpResponse {
    fn unauthorized(include_realms: bool) -> Self {
        (if include_realms {
            HttpResponse::new(StatusCode::UNAUTHORIZED)
                .with_header(header::WWW_AUTHENTICATE, "Bearer realm=\"Stalwart Server\"")
                .with_header(header::WWW_AUTHENTICATE, "Basic realm=\"Stalwart Server\"")
        } else {
            HttpResponse::new(StatusCode::UNAUTHORIZED)
        })
        .with_content_type("application/problem+json")
        .with_text_body(r#"{"type":"about:blank","status":401,"title":"Unauthorized","detail":"You have to authenticate first."}"#)
    }
}

impl ManagementApiError<'_> {
    fn into_http_response(self) -> HttpResponse {
        JsonResponse::new(self).into_http_response()
    }
}
