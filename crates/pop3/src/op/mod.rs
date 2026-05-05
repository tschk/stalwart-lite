/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::common::listener::SessionStream;

use crate::pop3::{
    Session,
    protocol::{Mechanism, response::Response},
};

pub mod authenticate;
pub mod delete;
pub mod fetch;
pub mod list;

impl<T: SessionStream> Session<T> {
    pub async fn handle_capa(&mut self) -> crate::trc::Result<()> {
        let mechanisms = if self.stream.is_tls() || self.server.core.imap.allow_plain_auth {
            vec![Mechanism::Plain, Mechanism::OAuthBearer, Mechanism::XOauth2]
        } else {
            vec![Mechanism::OAuthBearer, Mechanism::XOauth2]
        };

        crate::trc::event!(
            Pop3(crate::trc::Pop3Event::Capabilities),
            SpanId = self.session_id,
            Tls = self.stream.is_tls(),
            Strict = !self.server.core.imap.allow_plain_auth,
            Elapsed = crate::trc::Value::Duration(0)
        );

        self.write_bytes(
            Response::Capability::<u32> {
                mechanisms,
                stls: !self.stream.is_tls(),
            }
            .serialize(),
        )
        .await
    }

    pub async fn handle_stls(&mut self) -> crate::trc::Result<()> {
        crate::trc::event!(
            Pop3(crate::trc::Pop3Event::StartTls),
            SpanId = self.session_id,
            Elapsed = crate::trc::Value::Duration(0)
        );

        self.write_ok("Begin TLS negotiation now").await
    }

    pub async fn handle_utf8(&mut self) -> crate::trc::Result<()> {
        crate::trc::event!(
            Pop3(crate::trc::Pop3Event::Utf8),
            SpanId = self.session_id,
            Elapsed = crate::trc::Value::Duration(0)
        );

        self.write_ok("UTF8 enabled").await
    }
}
