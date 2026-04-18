use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    config::Config,
    models::{score::Score, score_history::ScoreHistory},
};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

#[derive(Deserialize)]
pub struct AddRequest {
    key: String, // max 32 chars
    value: f64,
}

#[derive(Deserialize)]
pub struct RemoveRequest {
    key: String, // max 32 chars
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    key: Option<String>, // max 32 chars
    start: Option<String>,
    end: Option<String>,
}

async fn add_handler(
    State(state): State<AppState>,
    Json(req): Json<AddRequest>,
) -> Result<Json<Score>, StatusCode> {
    let mut uploader: String = req.key;
    uploader.truncate(32);
    let value: f64 = req.value;

    Score::delete_by_uploader(&state.db, &uploader)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_score: Score = Score::create(&state.db, &uploader, value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    ScoreHistory::create(&state.db, uploader, value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(new_score))
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
        .route("/remove/{uploader}", delete(remove_handler))
        .route("/performance", get(performance_handler))
        .route("/info", get(info_handler))
        .route("/history", get(history_handler))
        .with_state(state)
}
