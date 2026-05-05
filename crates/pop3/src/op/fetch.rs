/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::common::listener::SessionStream;
use crate::directory::Permission;
use crate::email::message::metadata::MessageMetadata;
use crate::pop3::{Session, protocol::response::Response};
use crate::store::{
    ValueKey,
    write::{AlignedBytes, Archive},
};
use crate::trc::AddContext;
use crate::types::{collection::Collection, field::EmailField};
use crate::utils::chained_bytes::ChainedBytes;
use std::time::Instant;

impl<T: SessionStream> Session<T> {
    pub async fn handle_fetch(&mut self, msg: u32, lines: Option<u32>) -> crate::trc::Result<()> {
        // Validate access
        self.state
            .access_token()
            .assert_has_permission(Permission::Pop3Retr)?;

        let op_start = Instant::now();
        let mailbox = self.state.mailbox();
        if let Some(message) = mailbox.messages.get(msg.saturating_sub(1) as usize) {
            if let Some(metadata_) = self
                .server
                .store()
                .get_value::<Archive<AlignedBytes>>(ValueKey::property(
                    mailbox.account_id,
                    Collection::Email,
                    message.id,
                    EmailField::Metadata,
                ))
                .await
                .caused_by(crate::trc::location!())?
            {
                let metadata = metadata_
                    .unarchive::<MessageMetadata>()
                    .caused_by(crate::trc::location!())?;
                if let Some(bytes) = self
                    .server
                    .blob_store()
                    .get_blob(metadata.blob_hash.0.as_slice(), 0..usize::MAX)
                    .await
                    .caused_by(crate::trc::location!())?
                {
                    crate::trc::event!(
                        Pop3(crate::trc::Pop3Event::Fetch),
                        SpanId = self.session_id,
                        DocumentId = message.id,
                        Elapsed = op_start.elapsed()
                    );

                    let bytes = ChainedBytes::new(metadata.raw_headers.as_ref())
                        .with_last(
                            bytes
                                .get(metadata.blob_body_offset.to_native() as usize..)
                                .unwrap_or_default(),
                        )
                        .get_full_range();

                    self.write_bytes(
                        Response::Message::<u32> {
                            bytes,
                            lines: lines.unwrap_or(0),
                        }
                        .serialize(),
                    )
                    .await
                } else {
                    Err(crate::trc::Pop3Event::Error
                        .into_err()
                        .details("Failed to fetch message. Perhaps another session deleted it?")
                        .caused_by(crate::trc::location!()))
                }
            } else {
                Err(crate::trc::Pop3Event::Error
                    .into_err()
                    .details("Failed to fetch message. Perhaps another session deleted it?")
                    .caused_by(crate::trc::location!()))
            }
        } else {
            Err(crate::trc::Pop3Event::Error
                .into_err()
                .details("No such message."))
        }
    }
}
