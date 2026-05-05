/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::imap_proto::{Command, receiver::Request};

use crate::common::listener::SessionStream;
use crate::imap::core::Session;
use mail_send::Credentials;

impl<T: SessionStream> Session<T> {
    pub async fn handle_login(&mut self, request: Request<Command>) -> crate::trc::Result<()> {
        let arguments = request.parse_login()?;

        self.authenticate(
            Credentials::Plain {
                username: arguments.username.to_string(),
                secret: arguments.password.to_string(),
            },
            arguments.tag,
        )
        .await
    }
}
