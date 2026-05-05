/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::time::Instant;

use crate::common::listener::SessionStream;
use crate::directory::Permission;
use crate::imap_proto::receiver::Request;

use crate::managesieve::core::{Command, Session, StatusResponse};

impl<T: SessionStream> Session<T> {
    pub async fn handle_checkscript(
        &mut self,
        request: Request<Command>,
    ) -> crate::trc::Result<Vec<u8>> {
        // Validate access
        self.assert_has_permission(Permission::SieveCheckScript)?;

        let op_start = Instant::now();

        if request.tokens.is_empty() {
            return Err(crate::trc::ManageSieveEvent::Error
                .into_err()
                .details("Expected script as a parameter."));
        }

        let script = request.tokens.into_iter().next().unwrap().unwrap_bytes();
        self.server
            .core
            .sieve
            .untrusted_compiler
            .compile(&script)
            .map(|_| {
                crate::trc::event!(
                    ManageSieve(crate::trc::ManageSieveEvent::CheckScript),
                    SpanId = self.session_id,
                    Size = script.len(),
                    Elapsed = op_start.elapsed()
                );

                StatusResponse::ok("Script is valid.").into_bytes()
            })
            .map_err(|err| {
                crate::trc::ManageSieveEvent::Error
                    .into_err()
                    .details(err.to_string())
            })
    }
}
