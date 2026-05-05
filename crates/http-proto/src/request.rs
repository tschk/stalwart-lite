/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use std::borrow::Cow;

use compact_str::ToCompactString;
use http_body_util::BodyExt;

use crate::http_proto::HttpRequest;

#[inline]
pub fn decode_path_element(item: &str) -> Cow<'_, str> {
    percent_encoding::percent_decode_str(item)
        .decode_utf8()
        .unwrap_or_else(|_| item.into())
}

pub async fn fetch_body(
    req: &mut HttpRequest,
    max_size: usize,
    session_id: u64,
) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(1024);
    while let Some(Ok(frame)) = req.frame().await {
        if let Some(data) = frame.data_ref() {
            if bytes.len() + data.len() <= max_size || max_size == 0 {
                bytes.extend_from_slice(data);
            } else {
                crate::trc::event!(
                    Http(crate::trc::HttpEvent::RequestBody),
                    SpanId = session_id,
                    Details = req
                        .headers()
                        .iter()
                        .map(|(k, v)| crate::trc::Value::Array(vec![
                            k.as_str().to_compact_string().into(),
                            v.to_str().unwrap_or_default().to_compact_string().into()
                        ]))
                        .collect::<Vec<_>>(),
                    Contents = std::str::from_utf8(&bytes)
                        .unwrap_or("[binary data]")
                        .to_string(),
                    Size = bytes.len(),
                    Limit = max_size,
                );

                return None;
            }
        }
    }

    crate::trc::event!(
        Http(crate::trc::HttpEvent::RequestBody),
        SpanId = session_id,
        Details = req
            .headers()
            .iter()
            .map(|(k, v)| crate::trc::Value::Array(vec![
                k.as_str().to_compact_string().into(),
                v.to_str().unwrap_or_default().to_compact_string().into()
            ]))
            .collect::<Vec<_>>(),
        Contents = std::str::from_utf8(&bytes)
            .unwrap_or("[binary data]")
            .to_string(),
        Size = bytes.len(),
    );

    bytes.into()
}
