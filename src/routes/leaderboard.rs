use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use crate::{models::score::Score, router::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_leaderboard))
        .route("/json", get(get_leaderboard_json))
        .route("/{num}", get(get_leaderboard_num))
        .route("/json/{num}", get(get_leaderboard_json_num))
}

/// Returns a pretty-print version of the top 10 on the leaderboard
pub async fn get_leaderboard(State(_state): State<AppState>) {
    todo!("Basic pretty print")
}

/// Returns a json version of the top 10 on the leaderboard
pub async fn get_leaderboard_json(
    State(state): State<AppState>,
) -> Result<Json<Vec<Score>>, StatusCode> {
    Ok(Json(
        Score::leaderboard(&state.db, state.config.read().await.leaderboard.sort_order)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

/// Returns a pretty-print version of the top num on the leaderboard
pub async fn get_leaderboard_num(
    State(_state): State<AppState>,
    Path(_num): Path<i32>,
) -> Result<Json<Vec<Score>>, StatusCode> {
    todo!("Num pretty print")
}

/// Returns a json version of the top num on the leaderboard
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
