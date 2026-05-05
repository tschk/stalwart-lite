/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::time::Instant;

use crate::common::listener::SessionStream;
use crate::directory::Permission;
use crate::imap_proto::receiver::Request;
use crate::store::{SerializeInfallible, write::BatchBuilder};
use crate::trc::AddContext;
use crate::types::{collection::Collection, field::PrincipalField};

use crate::managesieve::core::{Command, Session, StatusResponse};

impl<T: SessionStream> Session<T> {
    pub async fn handle_setactive(
        &mut self,
        request: Request<Command>,
    ) -> crate::trc::Result<Vec<u8>> {
        // Validate access
        self.assert_has_permission(Permission::SieveSetActive)?;

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

        // De/activate script
        let account_id = self.state.access_token().primary_id();
        let mut batch = BatchBuilder::new();
        if !name.is_empty() {
            let document_id = self.get_script_id(account_id, &name).await?;
            batch
                .with_account_id(account_id)
                .with_collection(Collection::Principal)
                .with_document(0)
                .set(PrincipalField::ActiveScriptId, document_id.serialize());
        } else {
            batch
                .with_account_id(account_id)
                .with_collection(Collection::Principal)
                .with_document(0)
                .clear(PrincipalField::ActiveScriptId);
        }
        self.server
            .commit_batch(batch)
            .await
            .caused_by(crate::trc::location!())?;

        crate::trc::event!(
            ManageSieve(crate::trc::ManageSieveEvent::SetActive),
            SpanId = self.session_id,
            Id = name,
            Elapsed = op_start.elapsed()
        );

        Ok(StatusResponse::ok("Success").into_bytes())
    }
}
