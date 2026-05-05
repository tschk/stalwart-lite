/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::time::Instant;

use crate::common::listener::SessionStream;
use crate::directory::Permission;
use crate::imap_proto::receiver::Request;
use crate::trc::AddContext;

use crate::managesieve::core::{Command, ResponseCode, Session, StatusResponse};

impl<T: SessionStream> Session<T> {
    pub async fn handle_havespace(
        &mut self,
        request: Request<Command>,
    ) -> crate::trc::Result<Vec<u8>> {
        // Validate access
        self.assert_has_permission(Permission::SieveHaveSpace)?;

        let op_start = Instant::now();
        let mut tokens = request.tokens.into_iter();
        let name = tokens
            .next()
            .and_then(|s| s.unwrap_string().ok())
            .ok_or_else(|| {
                crate::trc::ManageSieveEvent::Error
                    .into_err()
                    .details("Expected script name as a parameter.")
            })?;
        let size: usize = tokens
            .next()
            .and_then(|s| s.unwrap_string().ok())
            .ok_or_else(|| {
                crate::trc::ManageSieveEvent::Error
                    .into_err()
                    .details("Expected script size as a parameter.")
            })?
            .parse::<usize>()
            .map_err(|_| {
                crate::trc::ManageSieveEvent::Error
                    .into_err()
                    .details("Invalid size parameter.")
            })?;

        // Validate name
        let access_token = self.state.access_token();
        let account_id = access_token.primary_id();
        self.validate_name(account_id, &name).await?;

        // Validate quota
        if access_token.quota == 0
            || size as i64
                + self
                    .server
                    .get_used_quota(account_id)
                    .await
                    .caused_by(crate::trc::location!())?
                <= access_token.quota as i64
        {
            crate::trc::event!(
                ManageSieve(crate::trc::ManageSieveEvent::HaveSpace),
                SpanId = self.session_id,
                Size = size,
                Elapsed = op_start.elapsed()
            );

            Ok(StatusResponse::ok("").into_bytes())
        } else {
            Err(crate::trc::ManageSieveEvent::Error
                .into_err()
                .details("Quota exceeded.")
                .code(ResponseCode::QuotaMaxSize))
        }
    }
}
