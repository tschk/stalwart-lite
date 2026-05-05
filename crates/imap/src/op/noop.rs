/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::time::Instant;

use crate::common::listener::SessionStream;
use crate::imap::core::{Session, State};
use crate::imap_proto::{Command, StatusResponse, receiver::Request};

impl<T: SessionStream> Session<T> {
    pub async fn handle_noop(&mut self, request: Request<Command>) -> crate::trc::Result<()> {
        let op_start = Instant::now();

        if let State::Selected { data, mailbox, .. } = &self.state {
            data.write_changes(
                &Some(mailbox.clone()),
                false,
                true,
                self.is_qresync,
                self.version.is_rev2(),
                self.is_utf8,
            )
            .await?;
        }

        crate::trc::event!(
            Imap(crate::trc::ImapEvent::Noop),
            SpanId = self.session_id,
            Elapsed = op_start.elapsed()
        );

        self.write_bytes(
            StatusResponse::completed(request.command)
                .with_tag(request.tag)
                .into_bytes(),
        )
        .await
    }
}
