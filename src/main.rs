use anyhow::Result;
use axum::Server;
use std::net::SocketAddr;
use tracing::info;

use lcore_node::{
    api,
    config::Config,
    database,
    AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting lcore-node...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded.");

    // Set up database connection pool
    // let db_pool = database::create_pool(&config.database_url).await?;
    // info!("Database pool created.");

    // Create application state
    let app_state = AppState {};

    // Create the Axum server
    let app = api::create_router(app_state);
    let addr: SocketAddr = config.server_addr.parse()?;
    info!("Starting server on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    info!("lcore-node shutting down.");

    Ok(())
} 