use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};

use crate::{models::score_history::ScoreHistory, router::AppState};

/// TODO
pub async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ScoreHistory>>, StatusCode> {
    let count: i64 = params
        .get("count")
        .unwrap_or(&String::from("5"))
        .parse()
        .unwrap_or(5);
    let page: i64 = params
        .get("page")
        .unwrap_or(&String::from("1"))
        .parse()
        .unwrap_or(1);
    Ok(Json(
        ScoreHistory::all(&state.db, count, page)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

/// TODO
pub async fn user_history(
    State(state): State<AppState>,
    Path(user): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ScoreHistory>>, StatusCode> {
    let count: i64 = params
        .get("count")
        .unwrap_or(&String::from("5"))
        .parse()
        .unwrap_or(5);
    let page: i64 = params
        .get("page")
        .unwrap_or(&String::from("1"))
        .parse()
        .unwrap_or(1);
    Ok(Json(
        ScoreHistory::from_user(&state.db, user, count, page)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_history))
        .route("/{user}", get(user_history))
}
