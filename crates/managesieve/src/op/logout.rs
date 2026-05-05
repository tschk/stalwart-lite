/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use tokio::io::{AsyncRead, AsyncWrite};

use crate::managesieve::core::{Session, StatusResponse};

impl<T: AsyncRead + AsyncWrite> Session<T> {
    pub async fn handle_logout(&mut self) -> crate::trc::Result<Vec<u8>> {
        crate::trc::event!(
            ManageSieve(crate::trc::ManageSieveEvent::Logout),
            SpanId = self.session_id,
            Elapsed = crate::trc::Value::Duration(0)
        );

        Ok(StatusResponse::ok("Stalwart ManageSieve bids you farewell.").into_bytes())
    }
}
