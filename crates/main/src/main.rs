/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

#![warn(clippy::large_futures)]
#![warn(clippy::cast_possible_truncation)]
#![warn(clippy::cast_possible_wrap)]
#![warn(clippy::cast_sign_loss)]

use stalwart_lib as trc;
use stalwart_lib::Collector;
use stalwart_lib::common::{
    config::server::ServerProtocol, core::BuildServer, manager::boot::BootManager,
};
use stalwart_lib::http::HttpSessionManager;
use stalwart_lib::imap::core::ImapSessionManager;
use stalwart_lib::services::{StartServices, broadcast::subscriber::spawn_broadcast_subscriber};
use stalwart_lib::smtp::StartQueueManager;
use stalwart_lib::utils::wait_for_shutdown;
use std::time::Duration;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load config and apply macros
    let mut init = Box::pin(BootManager::init()).await;

    // Migrate database
    if let Err(err) = stalwart_lib::migration::try_migrate(&init.inner.build_server()).await {
        trc::event!(
            Server(trc::ServerEvent::StartupError),
            Details = "Failed to migrate database, aborting startup.",
            Reason = err,
        );
        return Ok(());
    }

    // Init services
    init.start_services().await;
    init.start_queue_manager();

    // Log configuration errors
    init.config.log_errors();
    init.config.log_warnings();

    // SPDX-SnippetBegin
    // SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
    // SPDX-License-Identifier: LicenseRef-SEL
    #[cfg(feature = "enterprise")]
    init.inner.build_server().log_license_details();
    // SPDX-SnippetEnd

    // Fail fast if any configured listener uses an unsupported protocol
    for server in &init.servers.servers {
        if !matches!(
            server.protocol,
            ServerProtocol::Smtp
                | ServerProtocol::Lmtp
                | ServerProtocol::Http
                | ServerProtocol::Imap
        ) {
            trc::event!(
                Server(trc::ServerEvent::StartupError),
                Details = "Unsupported protocol in config — aborting startup. \
                           Only SMTP, LMTP, HTTP, and IMAP are supported in this build.",
                Reason = format!("{:?}", server.protocol),
            );
            return Ok(());
        }
    }

    // Spawn servers
    let (shutdown_tx, shutdown_rx) = init.servers.spawn(|server, acceptor, shutdown_rx| {
        match &server.protocol {
            ServerProtocol::Smtp | ServerProtocol::Lmtp => server.spawn(
                stalwart_lib::smtp::core::SmtpSessionManager::new(init.inner.clone()),
                init.inner.clone(),
                acceptor,
                shutdown_rx,
            ),
            ServerProtocol::Http => server.spawn(
                HttpSessionManager::new(init.inner.clone()),
                init.inner.clone(),
                acceptor,
                shutdown_rx,
            ),
            ServerProtocol::Imap => server.spawn(
                ImapSessionManager::new(init.inner.clone()),
                init.inner.clone(),
                acceptor,
                shutdown_rx,
            ),
            _ => unreachable!("pre-validated above"),
        };
    });

    // Start broadcast subscriber
    spawn_broadcast_subscriber(init.inner, shutdown_rx);

    // Wait for shutdown signal
    wait_for_shutdown().await;

    // Shutdown collector
    Collector::shutdown();

    // Stop services
    let _ = shutdown_tx.send(true);

    // Wait for services to finish
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
