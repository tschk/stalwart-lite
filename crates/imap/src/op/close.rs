/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::time::Instant;

use crate::common::listener::SessionStream;
use crate::imap::core::{Session, State};
use crate::imap_proto::{Command, StatusResponse, receiver::Request};
use crate::trc::AddContext;

impl<T: SessionStream> Session<T> {
    pub async fn handle_close(&mut self, request: Request<Command>) -> crate::trc::Result<()> {
        let op_start = Instant::now();
        let (data, mailbox) = self.state.select_data();

        if mailbox.is_select {
            data.expunge(mailbox.clone(), None, op_start)
                .await
                .caused_by(crate::trc::location!())?;
        }

        crate::trc::event!(
            Imap(crate::trc::ImapEvent::Close),
            SpanId = self.session_id,
            AccountId = mailbox.id.account_id,
            MailboxId = mailbox.id.mailbox_id,
            Elapsed = op_start.elapsed()
        );

        self.state = State::Authenticated { data };
        self.write_bytes(
            StatusResponse::completed(Command::Close)
                .with_tag(request.tag)
                .into_bytes(),
        )
        .await
    }
}
