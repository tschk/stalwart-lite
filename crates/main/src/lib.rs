/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

//! Stalwart Mail Server — public library API
//!
//! Provides an embeddable entry-point for the IMAP+SMTP mail server.

#[path = "../../common/src/lib.rs"]
pub mod common;

#[path = "../../store/src/lib.rs"]
pub mod store;

#[path = "../../directory/src/lib.rs"]
pub mod directory;

#[path = "../../email/src/lib.rs"]
pub mod email;

#[path = "../../http/src/lib.rs"]
pub mod http;

#[path = "../../imap/src/lib.rs"]
pub mod imap;

#[path = "../../smtp/src/lib.rs"]
pub mod smtp;

#[path = "../../services/src/lib.rs"]
pub mod services;

#[path = "../../types/src/lib.rs"]
pub mod types;

#[path = "../../utils/src/lib.rs"]
pub mod utils;

extern crate self as trc;

#[path = "../../trc/src/lib.rs"]
pub mod trc_impl;

#[path = "../../migration/src/lib.rs"]
pub mod migration;

#[path = "../../nlp/src/lib.rs"]
pub mod nlp;

#[path = "../../spam-filter/src/lib.rs"]
pub mod spam_filter;

#[path = "../../http-proto/src/lib.rs"]
pub mod http_proto;

#[path = "../../imap-proto/src/lib.rs"]
pub mod imap_proto;

#[path = "../../jmap-proto/src/lib.rs"]
pub mod jmap_proto;

#[path = "../../groupware/src/lib.rs"]
pub mod groupware;

#[path = "../../dav-proto/src/lib.rs"]
pub mod dav_proto;

#[path = "../../jmap/src/lib.rs"]
pub mod jmap;

#[path = "../../dav/src/lib.rs"]
pub mod dav;

#[path = "../../pop3/src/lib.rs"]
pub mod pop3;

#[path = "../../managesieve/src/lib.rs"]
pub mod managesieve;

pub use crate::common::{Server, manager::boot::BootManager};
pub use crate::trc_impl::*;

/// Start the server and return a [`Server`] handle.
///
/// Configuration is read from the `CONFIG_PATH` environment variable, or
/// defaults to `/etc/stalwart/config.toml` if the variable is not set.
///
/// # Process exit
///
/// Internally this calls [`BootManager::init`], which prints diagnostics and
/// calls [`std::process::exit`] if the config file is missing or fatally
/// invalid. This is intentional for a server binary but may be surprising
/// when embedded as a library. Make sure a valid config file exists at the
/// expected path before calling this function.
pub async fn start_server() -> std::io::Result<Server> {
    use crate::common::core::BuildServer;
    let init = Box::pin(BootManager::init()).await;
    Ok(init.inner.build_server())
}
