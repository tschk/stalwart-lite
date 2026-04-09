/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

//! Stalwart Mail Server — public library API
//!
//! Provides an embeddable entry-point for the IMAP+SMTP mail server.

pub use common::{Server, manager::boot::BootManager};

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
    use common::core::BuildServer;
    let init = Box::pin(BootManager::init()).await;
    Ok(init.inner.build_server())
}
