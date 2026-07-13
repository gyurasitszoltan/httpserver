mod app;
mod auth;
mod clock;
mod config;
mod crypto;
mod db;
mod dto;
mod error;
mod mail;
mod routes;
mod state;
mod validation;

use tracing::info;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::load().map_err(std::io::Error::other)?;
    let app = app::build(&config).await?;
    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    info!(address = %config.addr, "szerver elindult");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
