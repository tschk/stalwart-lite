/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::common::listener::SessionStream;
use crate::directory::Permission;
use crate::email::sieve::{delete::SieveScriptDelete, ingest::SieveScriptIngest};
use crate::imap_proto::receiver::Request;
use crate::managesieve::core::{Command, ResponseCode, Session, StatusResponse};
use crate::store::write::BatchBuilder;
use crate::trc::AddContext;
use std::time::Instant;

impl<T: SessionStream> Session<T> {
    pub async fn handle_deletescript(
        &mut self,
        request: Request<Command>,
    ) -> crate::trc::Result<Vec<u8>> {
        // Validate access
        self.assert_has_permission(Permission::SieveDeleteScript)?;

        let op_start = Instant::now();

        let name = request
            .tokens
            .into_iter()
            .next()
            .and_then(|s| s.unwrap_string().ok())
            .ok_or_else(|| {
                crate::trc::ManageSieveEvent::Error
                    .into_err()
                    .details("Expected script name as a parameter.")
            })?;

        let access_token = self.state.access_token();
        let account_id = access_token.primary_id();
        let document_id = self.get_script_id(account_id, &name).await?;
        let mut batch = BatchBuilder::new();

        let active_script_id = self.server.sieve_script_get_active_id(account_id).await?;
        if active_script_id != Some(document_id) {
            if self
                .server
                .sieve_script_delete(account_id, document_id, access_token, &mut batch)
                .await
                .caused_by(crate::trc::location!())?
            {
                if !batch.is_empty() {
                    self.server
                        .commit_batch(batch)
                        .await
                        .caused_by(crate::trc::location!())?;
                }

                crate::trc::event!(
                    ManageSieve(crate::trc::ManageSieveEvent::DeleteScript),
                    SpanId = self.session_id,
                    Id = name,
                    DocumentId = document_id,
                    Elapsed = op_start.elapsed()
                );

                Ok(StatusResponse::ok("Deleted.").into_bytes())
            } else {
                Err(crate::trc::ManageSieveEvent::Error
                    .into_err()
                    .details("Script not found"))
            }
        } else {
            Err(crate::trc::ManageSieveEvent::Error
                .into_err()
                .details("You may not delete an active script")
                .code(ResponseCode::Active))
        }
    }
}
