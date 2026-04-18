use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::{Config, LeaderboardSortOrder},
    models::{
        score::{Score, ScoreStats},
        score_history::ScoreHistory,
    },
    routes,
};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<RwLock<Config>>,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: String,
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

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    ),
    tag = "health"
)]
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
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

async fn info_handler(State(state): State<AppState>) -> Result<Json<ScoreStats>, StatusCode> {
    Ok(Json(
        Score::get_score_stats(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn history_handler(State(_state): State<AppState>, Query(_params): Query<HistoryQuery>) {
    todo!()
}

#[derive(Serialize)]
struct BoardConfigResponse {
    title: String,
    sort_order: LeaderboardSortOrder,
}

async fn board_config_handler(State(state): State<AppState>) -> Json<BoardConfigResponse> {
    let config = state.config.read().await;
    Json(BoardConfigResponse {
        title: config.leaderboard.title.clone(),
        sort_order: config.leaderboard.sort_order,
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(health_handler),
    components(schemas(HealthResponse)),
    tags(
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;

pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/add", post(add_handler))
        .route("/remove/{uploader}", delete(remove_handler))
        .route("/performance", get(performance_handler))
        .route("/info", get(info_handler))
        .route("/history", get(history_handler))
        .route("/boardconfig", get(board_config_handler))
        .nest("/admin", routes::admin::router(state.clone()))
        .nest("/leaderboard", routes::leaderboard::router())
        .layer(cors)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
