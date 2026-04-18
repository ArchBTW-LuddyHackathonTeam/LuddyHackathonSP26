use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use tabled::{settings::Style, Table};
use utoipa::IntoParams;

use crate::{models::score::Score, router::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_leaderboard))
        .route("/json", get(get_leaderboard_json))
        .route("/{num}", get(get_leaderboard_num))
        .route("/json/{num}", get(get_leaderboard_json_num))
}

/// Path parameter for leaderboard size.
#[derive(IntoParams)]
struct NumParam {
    /// Maximum number of entries to return.
    num: i32,
}

/// Get the top-10 leaderboard as a Markdown table.
///
/// Returns a human-readable Markdown-formatted table of the top 10 scores,
/// sorted according to the configured sort order.
#[utoipa::path(
    get,
    path = "/leaderboard",
    responses(
        (status = 200, description = "Markdown table of the top-10 scores", content_type = "text/plain"),
        (status = 500, description = "Database error")
    ),
    tag = "leaderboard"
)]
pub async fn get_leaderboard(State(state): State<AppState>) -> Result<String, StatusCode> {
    let scores: Vec<Score> =
        Score::leaderboard(&state.db, state.config.read().await.leaderboard.sort_order)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut table: Table = Table::new(scores);
    table.with(Style::markdown());

    Ok(table.to_string())
}

/// Get the top-10 leaderboard as JSON.
///
/// Returns the top 10 scores as a JSON array, sorted according to the
/// configured sort order.
#[utoipa::path(
    get,
    path = "/leaderboard/json",
    responses(
        (status = 200, description = "JSON array of top-10 scores", body = Vec<Score>),
        (status = 500, description = "Database error")
    ),
    tag = "leaderboard"
)]
pub async fn get_leaderboard_json(
    State(state): State<AppState>,
) -> Result<Json<Vec<Score>>, StatusCode> {
    Ok(Json(
        Score::leaderboard(&state.db, state.config.read().await.leaderboard.sort_order)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

/// Get the top-N leaderboard as a Markdown table.
///
/// Returns a human-readable Markdown-formatted table of the top `num` scores,
/// sorted according to the configured sort order.
#[utoipa::path(
    get,
    path = "/leaderboard/{num}",
    params(
        ("num" = i32, Path, description = "Maximum number of leaderboard entries to return")
    ),
    responses(
        (status = 200, description = "Markdown table of the top-N scores", content_type = "text/plain"),
        (status = 500, description = "Database error")
    ),
    tag = "leaderboard"
)]
pub async fn get_leaderboard_num(
    State(state): State<AppState>,
    Path(num): Path<i32>,
) -> Result<String, StatusCode> {
    let scores: Vec<Score> = Score::leaderboard_num(
        &state.db,
        num,
        state.config.read().await.leaderboard.sort_order,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut table: Table = Table::new(scores);
    table.with(Style::markdown());

    Ok(table.to_string())
}

/// Get the top-N leaderboard as JSON.
///
/// Returns the top `num` scores as a JSON array, sorted according to the
/// configured sort order.
#[utoipa::path(
    get,
    path = "/leaderboard/json/{num}",
    params(
        ("num" = i32, Path, description = "Maximum number of leaderboard entries to return")
    ),
    responses(
        (status = 200, description = "JSON array of top-N scores", body = Vec<Score>),
        (status = 500, description = "Database error")
    ),
    tag = "leaderboard"
)]
pub async fn get_leaderboard_json_num(
    State(state): State<AppState>,
    Path(num): Path<i32>,
) -> Result<Json<Vec<Score>>, StatusCode> {
    Ok(Json(
        Score::leaderboard_num(
            &state.db,
            num,
            state.config.read().await.leaderboard.sort_order,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}
