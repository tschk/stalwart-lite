/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

//! Stalwart Mail Server — public library API
//!
//! Provides simple functions to store, send, and retrieve email
//! through the embedded JMAP server.

pub use common::{Server, manager::boot::BootManager};

/// Start the server and return a handle. Config path is read from
/// the `CONFIG_PATH` env var or defaults to `/etc/stalwart/config.toml`.
pub async fn start_server() -> std::io::Result<Server> {
    use common::core::BuildServer;
    let init = Box::pin(BootManager::init()).await;
    Ok(init.inner.build_server())
}
