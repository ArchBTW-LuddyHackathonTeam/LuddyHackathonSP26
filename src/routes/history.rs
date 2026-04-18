use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};

use crate::{models::score_history::ScoreHistory, router::AppState};

pub async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ScoreHistory>>, StatusCode> {
    let count: i64 = params.get("count").and_then(|v| v.parse().ok()).unwrap_or(5);
    let page: i64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let title = params.get("title").map(|s| s.as_str());
    let start = params.get("start").map(|s| s.as_str());
    let end = params.get("end").map(|s| s.as_str());
    Ok(Json(
        ScoreHistory::query(&state.db, title, start, end, count, page)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn user_history(
    State(state): State<AppState>,
    Path(user): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ScoreHistory>>, StatusCode> {
    let count: i64 = params.get("count").and_then(|v| v.parse().ok()).unwrap_or(5);
    let page: i64 = params.get("page").and_then(|v| v.parse().ok()).unwrap_or(1);
    let start = params.get("start").map(|s| s.as_str());
    let end = params.get("end").map(|s| s.as_str());
    Ok(Json(
        ScoreHistory::query(&state.db, Some(&user), start, end, count, page)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_history))
        .route("/{user}", get(user_history))
}
