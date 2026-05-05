/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::Server;
use crate::types::{blob::BlobSection, blob_hash::BlobHash};
use mail_parser::{
    Encoding,
    decoders::{base64::base64_decode, quoted_printable::quoted_printable_decode},
};

impl Server {
    pub async fn get_blob_section(
        &self,
        hash: &BlobHash,
        section: &BlobSection,
    ) -> crate::trc::Result<Option<Vec<u8>>> {
        Ok(self
            .blob_store()
            .get_blob(
                hash.as_slice(),
                (section.offset_start)..(section.offset_start.saturating_add(section.size)),
            )
            .await?
            .and_then(|bytes| match Encoding::from(section.encoding) {
                Encoding::None => Some(bytes),
                Encoding::Base64 => base64_decode(&bytes),
                Encoding::QuotedPrintable => quoted_printable_decode(&bytes),
            }))
    }
}
