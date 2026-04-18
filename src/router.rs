use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
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

#[derive(Clone, Default)]
pub struct EndpointMetrics {
    pub count: u64,
    pub total_ms: f64,
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<RwLock<Config>>,
    pub metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
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

#[derive(Serialize)]
struct PerformanceEntry {
    endpoint: String,
    avg_ms: f64,
    count: u64,
}

async fn performance_handler(State(state): State<AppState>) -> Json<Vec<PerformanceEntry>> {
    let metrics = state.metrics.read().await;
    let mut entries: Vec<PerformanceEntry> = metrics
        .iter()
        .map(|(path, m)| PerformanceEntry {
            endpoint: path.clone(),
            avg_ms: if m.count > 0 {
                m.total_ms / m.count as f64
            } else {
                0.0
            },
            count: m.count,
        })
        .collect();
    entries.sort_by(|a, b| a.endpoint.cmp(&b.endpoint));
    Json(entries)
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

async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    if path != "/performance" {
        let normalized = if path.starts_with("/remove/") {
            "/remove".to_string()
        } else if path.starts_with("/leaderboard/json") {
            "/leaderboard/json".to_string()
        } else if path.starts_with("/leaderboard/") {
            "/leaderboard".to_string()
        } else {
            path
        };

        let mut metrics = state.metrics.write().await;
        let entry = metrics.entry(normalized).or_default();
        entry.count += 1;
        entry.total_ms += elapsed;
    }

    response
}

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
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            metrics_middleware,
        ))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
