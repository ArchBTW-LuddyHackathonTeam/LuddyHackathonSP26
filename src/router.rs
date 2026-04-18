use axum::{Router, routing::{get, post}, extract::{State, Query}, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

#[derive(Deserialize)]
pub struct AddRequest {
    key: String,  // max 32 chars
    value: f64,
}

#[derive(Deserialize)]
pub struct RemoveRequest {
    key: String,  // max 32 chars
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    key: Option<String>,  // max 32 chars
    start: Option<String>,
    end: Option<String>,
}

async fn add_handler(State(_state): State<AppState>, Json(_req): Json<AddRequest>) {
    todo!()
}

async fn remove_handler(State(_state): State<AppState>, Json(_req): Json<RemoveRequest>) {
    todo!()
}

async fn performance_handler(State(_state): State<AppState>) {
    todo!()
}

async fn info_handler(State(_state): State<AppState>) {
    todo!()
}

async fn history_handler(State(_state): State<AppState>, Query(_params): Query<HistoryQuery>) {
    todo!()
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/add", post(add_handler))
        .route("/remove", post(remove_handler))
        .route("/performance", get(performance_handler))
        .route("/info", get(info_handler))
        .route("/history", get(history_handler))
        .with_state(state)
}
