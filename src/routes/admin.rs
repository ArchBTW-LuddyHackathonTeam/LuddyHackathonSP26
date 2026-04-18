use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::patch,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    config::{Config, LeaderboardSortOrder},
    models::token::Token,
    router::AppState,
};

#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    title: Option<String>,
    sort_order: Option<LeaderboardSortOrder>,
}

async fn update_config_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<Config>, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.strip_prefix("Bearer ").unwrap_or(s))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mut hasher = Sha256::new();
    hasher.update(token);
    let hashed = hex::encode(hasher.finalize());

    if !Token::exists(&state.db, hashed)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut config = state.config.write().await;

    if let Some(title) = req.title {
        config.leaderboard.title = title;
    }

    if let Some(sort_order) = req.sort_order {
        if sort_order != LeaderboardSortOrder::Ascending
            && sort_order != LeaderboardSortOrder::Descending
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        config.leaderboard.sort_order = sort_order;
    }

    config
        .save()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(config.clone()))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/config", patch(update_config_handler))
}
