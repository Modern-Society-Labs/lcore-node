use axum::{routing::get, Router};
use crate::AppState;

pub fn create_router(_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        // More routes will be added here
}

async fn health_check() -> &'static str {
    "OK"
} 