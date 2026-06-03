use std::net::SocketAddr;

use anyhow::Context;
use promptops_arena_backend::{app, config::AppConfig, tracing};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env().context("failed to load configuration")?;
    tracing::init(&config.log.filter);

    let address = SocketAddr::from((config.server.host, config.server.port));
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind HTTP listener to {address}"))?;

    ::tracing::info!(%address, "starting HTTP server");
    axum::serve(listener, app::router())
        .await
        .context("HTTP server failed")?;

    Ok(())
}
