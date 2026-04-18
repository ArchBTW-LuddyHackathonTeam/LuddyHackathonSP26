use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    models::{score::Score, score_history::ScoreHistory},
    routes,
};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

#[derive(Deserialize)]
pub struct AddRequest {
    key: String,
    value: f64,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    key: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

async fn add_handler(
    State(state): State<AppState>,
    Json(req): Json<AddRequest>,
) -> Result<Json<Score>, StatusCode> {
    let mut uploader = req.key;
    uploader.truncate(32);

    Score::delete_by_uploader(&state.db, &uploader)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_score = Score::create(&state.db, &uploader, req.value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    ScoreHistory::create(&state.db, uploader, req.value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(new_score))
}

async fn remove_handler(State(state): State<AppState>, Path(uploader): Path<String>) -> StatusCode {
    let mut uploader: String = uploader;
    uploader.truncate(32);

    match Score::delete_by_uploader(&state.db, &uploader).await {
        Ok(value) => match value {
            0 => StatusCode::NOT_FOUND,
            1 => StatusCode::OK,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
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

#[derive(Serialize)]
struct BoardNameResponse {
    title: String,
}

#[derive(Serialize)]
struct BoardConfigResponse {
    title: String,
    sort_order: String,
}

async fn board_name_handler(State(state): State<AppState>) -> Json<BoardNameResponse> {
    Json(BoardNameResponse {
        title: state.config.leaderboard.title.clone(),
    })
}

async fn board_config_handler(State(state): State<AppState>) -> Json<BoardConfigResponse> {
    let config = state.config;
    Json(BoardConfigResponse {
        title: config.leaderboard.title.clone(),
        sort_order: config.leaderboard.sort_order,
    })
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/add", post(add_handler))
        .route("/remove/{uploader}", delete(remove_handler))
        .route("/performance", get(performance_handler))
        .route("/info", get(info_handler))
        .route("/history", get(history_handler))
        .route("/boardname", get(board_name_handler))
        .route("/boardconfig", get(board_config_handler))
        .nest("/admin", routes::admin::router())
        .with_state(state)
}
