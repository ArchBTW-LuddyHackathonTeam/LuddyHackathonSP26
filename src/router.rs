use axum::{routing::get, Router};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub secret: Uuid,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healt", get(|| async { "Health" }))
        .with_state(state)
}
