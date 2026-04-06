/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

#![warn(clippy::large_futures)]
#![warn(clippy::cast_possible_truncation)]
#![warn(clippy::cast_possible_wrap)]
#![warn(clippy::cast_sign_loss)]

use common::{config::server::ServerProtocol, core::BuildServer, manager::boot::BootManager};
use http::HttpSessionManager;
use services::{StartServices, broadcast::subscriber::spawn_broadcast_subscriber};
use smtp::StartQueueManager;
use std::time::Duration;
use trc::Collector;
use utils::wait_for_shutdown;

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
    if let Err(err) = migration::try_migrate(&init.inner.build_server()).await {
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

    // Spawn servers
    let (shutdown_tx, shutdown_rx) = init.servers.spawn(|server, acceptor, shutdown_rx| {
        match &server.protocol {
            ServerProtocol::Smtp | ServerProtocol::Lmtp => server.spawn(
                smtp::core::SmtpSessionManager::new(init.inner.clone()),
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
            _ => {
                trc::event!(
                    Server(trc::ServerEvent::StartupError),
                    Details = "Protocol not supported in this build",
                    Reason = format!("{:?}", server.protocol),
                );
            }
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
