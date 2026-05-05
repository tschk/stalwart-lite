/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::time::Instant;

use crate::common::listener::SessionStream;
use crate::imap::core::Session;
use crate::imap_proto::{Command, StatusResponse, receiver::Request};

impl<T: SessionStream> Session<T> {
    pub async fn handle_logout(&mut self, request: Request<Command>) -> crate::trc::Result<()> {
        let op_start = Instant::now();

        let mut response =
            StatusResponse::bye("Stalwart IMAP4rev2 bids you farewell.".to_string()).into_bytes();

        crate::trc::event!(
            Imap(crate::trc::ImapEvent::Logout),
            SpanId = self.session_id,
            Elapsed = op_start.elapsed()
        );

        response.extend(
            StatusResponse::completed(Command::Logout)
                .with_tag(request.tag)
                .into_bytes(),
        );
        self.write_bytes(response).await
    }
}
