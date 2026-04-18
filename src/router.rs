use axum::{Router, routing::get};
use uuid::Uuid;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
}
