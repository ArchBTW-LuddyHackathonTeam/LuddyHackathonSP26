use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use crate::{models::score_history::ScoreHistory, router::AppState};

/// Retrieve paginated score submission history across all participants.
///
/// Returns historical score submissions in reverse-chronological order
/// (most recent first), optionally filtered by a datetime range.
/// Results are paginated; use the `count` and `page` parameters to page
/// through large result sets.
#[utoipa::path(
    get,
    path = "/history",
    params(
        ("count" = i64, Query, description = "Number of items to display per page"),
        ("page" = i64, Query, description = "Page number of results"),
        ("start" = PrimitiveDateTime, Query, description = "Start timestamp of range"),
        ("end" = PrimitiveDateTime, Query, description = "End timestamp of range")
    ),
    responses(
        (status = 200, description = "Paginated list of score history entries", body = Vec<ScoreHistory>),
        (status = 500, description = "Database error")
    ),
    tag = "scores"
)]
pub async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ScoreHistory>>, StatusCode> {
    let count: i64 = params
        .get("count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
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

/// Retrieve paginated score submission history for a single participant.
///
/// Returns all historical score submissions for the given `user` in
/// reverse-chronological order (most recent first), optionally filtered
/// by a datetime range.  Results are paginated; use the `count` and
/// `page` parameters to page through large result sets.
#[utoipa::path(
    get,
    path = "/history/{user}",
    params(
        ("user" = String, Path, description = "Participant identifier to filter history by"),
        ("count" = i64, Query, description = "Number of items to display per page"),
        ("page" = i64, Query, description = "Page number of results"),
        ("start" = PrimitiveDateTime, Query, description = "Start timestamp of range"),
        ("end" = PrimitiveDateTime, Query, description = "End timestamp of range")
    ),
    responses(
        (status = 200, description = "Paginated list of score history entries for the given participant", body = Vec<ScoreHistory>),
        (status = 500, description = "Database error")
    ),
    tag = "scores"
)]
pub async fn user_history(
    State(state): State<AppState>,
    Path(user): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ScoreHistory>>, StatusCode> {
    let count: i64 = params
        .get("count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
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
