pub mod api;
pub mod config;
pub mod database;
pub mod encryption;
pub mod error;

/// The application state, which will be shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    // pub db_pool: sqlx::SqlitePool,
    // pub config: config::Config,
} 