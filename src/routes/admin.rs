use axum::{
    Json, Router,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::Response,
    routing::patch,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    config::{Config, LeaderboardSortOrder},
    models::token::Token,
    router::AppState,
};

/// Request body for updating the leaderboard configuration.
///
/// All fields are optional — only the supplied fields will be updated.
#[derive(Deserialize, ToSchema)]
pub struct UpdateConfigRequest {
    /// New human-readable title for the leaderboard.
    title: Option<String>,
    /// New sort order (`"ascending"` or `"descending"`).
    sort_order: Option<LeaderboardSortOrder>,
}

async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match request.headers().get(AUTHORIZATION) {
        Some(v) => match v.to_str() {
            Ok(token) => match Token::exists(
                &state.db,
                Token::hash(token.strip_prefix("Bearer ").unwrap_or(token)),
            )
            .await
            {
                Ok(true) => Ok(next.run(request).await),
                Ok(false) => Err(StatusCode::UNAUTHORIZED),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            },
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Update the leaderboard configuration.
///
/// Partially updates the leaderboard configuration.  Supply only the fields
/// you want to change.  Changes are persisted to `config.toml` immediately.
///
/// Requires a valid admin Bearer token in the `Authorization` header.
#[utoipa::path(
    patch,
    path = "/admin/config",
    request_body = UpdateConfigRequest,
    responses(
        (status = 200, description = "Configuration updated successfully", body = Config),
        (status = 400, description = "Invalid sort_order value supplied"),
        (status = 401, description = "Missing or invalid Authorization header"),
        (status = 500, description = "Database or filesystem error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "admin"
)]
pub async fn update_config_handler(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<Config>, StatusCode> {
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

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/config", patch(update_config_handler))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
