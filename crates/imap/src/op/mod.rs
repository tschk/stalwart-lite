/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use crate::imap_proto::ResponseCode;
use crate::store::query::log::Query;

pub mod acl;
pub mod append;
pub mod authenticate;
pub mod capability;
pub mod close;
pub mod copy_move;
pub mod create;
pub mod delete;
pub mod enable;
pub mod expunge;
pub mod fetch;
pub mod idle;
pub mod list;
pub mod login;
pub mod logout;
pub mod namespace;
pub mod noop;
pub mod quota;
pub mod rename;
pub mod search;
pub mod select;
pub mod status;
pub mod store;
pub mod subscribe;
pub mod thread;

trait FromModSeq {
    fn from_modseq(modseq: u64) -> Self;
}

trait ToModSeq {
    fn to_modseq(&self) -> u64;
}

impl FromModSeq for Query {
    fn from_modseq(modseq: u64) -> Self {
        if modseq > 0 {
            Query::Since(modseq - 1)
        } else {
            Query::All
        }
    }
}

impl ToModSeq for u64 {
    fn to_modseq(&self) -> u64 {
        if *self > 0 { *self + 1 } else { 0 }
    }
}

#[macro_export]
macro_rules! spawn_op {
    ($data:expr, $($code:tt)*) => {
        {

        tokio::spawn(async move {
            let data = &($data);

            if let Err(err) = (async {
                $($code)*
            })
            .await
            {
                let _ = data.write_error(err).await;
            }
        });

        Ok(())}
    };
}
pub trait ImapContext<T> {
    fn imap_ctx(self, tag: &str, location: &'static str) -> crate::trc::Result<T>;
}

impl<T> ImapContext<T> for crate::trc::Result<T> {
    fn imap_ctx(self, tag: &str, location: &'static str) -> crate::trc::Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(
                if !err.matches(crate::trc::EventType::Imap(crate::trc::ImapEvent::Error)) {
                    err.ctx(crate::trc::Key::Id, tag.to_string())
                        .ctx(crate::trc::Key::Details, "Internal Server Error")
                        .ctx(crate::trc::Key::Code, ResponseCode::ContactAdmin)
                        .ctx(crate::trc::Key::CausedBy, location)
                } else {
                    err.ctx(crate::trc::Key::Id, tag.to_string())
                },
            ),
        }
    }
}
