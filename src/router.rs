use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Request, State},
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

/// Response body for the health check endpoint.
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Always `"OK"` when the service is healthy.
    status: String,
}

/// Request body for submitting or updating a score.
#[derive(Deserialize, ToSchema)]
pub struct AddRequest {
    /// The participant's unique identifier (max 32 characters).
    key: String,
    /// The score value to record.
    value: f64,
}

/// A single entry in the per-endpoint performance report.
#[derive(Serialize, ToSchema)]
pub struct PerformanceEntry {
    /// The normalized endpoint path (e.g. `/add`, `/remove`, `/leaderboard/json`).
    endpoint: String,
    /// Average latency in milliseconds across all recorded requests.
    avg_ms: f64,
    /// Total number of requests recorded for this endpoint.
    count: u64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Check whether the service is up and running.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "health"
)]
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}

/// Submit or replace a participant's score.
///
/// If the participant (`key`) already has a score on the leaderboard it is
/// deleted before the new one is inserted, so each participant always has at
/// most one active score.  The old value is still preserved in the score
/// history.  The key is silently truncated to 32 characters.
#[utoipa::path(
    post,
    path = "/add",
    request_body = AddRequest,
    responses(
        (status = 200, description = "Score accepted and recorded", body = Score),
        (status = 500, description = "Database error")
    ),
    tag = "scores"
)]
async fn add_handler(
    State(state): State<AppState>,
    Json(req): Json<AddRequest>,
) -> Result<Json<Score>, StatusCode> {
    let mut uploader = req.key;
    uploader.truncate(32);

    let existing_score = Score::from_uploader(&state.db, &uploader)
        .await
        .map_err(|e| {
            eprintln!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // save existing to history before overwriting
    if let Some(old_score) = existing_score {
        ScoreHistory::create(&state.db, old_score.uploader.clone(), old_score.value)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let new_score = Score::create(&state.db, &uploader, req.value)
        .await
        .map_err(|e| {
            eprintln!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // save the new score to history
    ScoreHistory::create(&state.db, uploader, req.value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(new_score))
}

/// Remove a participant's score from the leaderboard.
///
/// Deletes the active score for the given `uploader`.  The score history is
/// **not** affected.  The uploader value is silently truncated to 32
/// characters.
#[utoipa::path(
    delete,
    path = "/remove/{uploader}",
    params(
        ("uploader" = String, Path, description = "Participant identifier (max 32 characters)")
    ),
    responses(
        (status = 200, description = "Score deleted successfully"),
        (status = 404, description = "No score found for the given uploader"),
        (status = 500, description = "Database error")
    ),
    tag = "scores"
)]
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

/// Retrieve per-endpoint latency and request-count metrics.
///
/// Returns an alphabetically sorted list of all endpoints that have received
/// at least one request since the server started.  The `/performance` endpoint
/// itself is excluded from tracking.
#[utoipa::path(
    get,
    path = "/performance",
    responses(
        (status = 200, description = "Endpoint performance metrics", body = Vec<PerformanceEntry>)
    ),
    tag = "metrics"
)]
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

/// Get aggregate statistics for all scores currently on the leaderboard.
///
/// Returns descriptive statistics including mean, median, standard deviation,
/// percentiles, and more.
#[utoipa::path(
    get,
    path = "/info",
    responses(
        (status = 200, description = "Score statistics", body = ScoreStats),
        (status = 500, description = "Database error")
    ),
    tag = "scores"
)]
async fn info_handler(State(state): State<AppState>) -> Result<Json<ScoreStats>, StatusCode> {
    Ok(Json(
        Score::get_score_stats(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

/// Response body describing the current leaderboard configuration.
#[derive(Serialize, ToSchema)]
pub struct BoardConfigResponse {
    /// Human-readable title of the leaderboard.
    title: String,
    /// Whether scores are ranked from highest-to-lowest or lowest-to-highest.
    sort_order: LeaderboardSortOrder,
}

/// Get the current leaderboard display configuration.
///
/// Returns the leaderboard title and sort order as configured in `config.toml`
/// (or updated via `PATCH /admin/config`).
#[utoipa::path(
    get,
    path = "/boardconfig",
    responses(
        (status = 200, description = "Current board configuration", body = BoardConfigResponse)
    ),
    tag = "config"
)]
async fn board_config_handler(State(state): State<AppState>) -> Json<BoardConfigResponse> {
    let config = state.config.read().await;
    Json(BoardConfigResponse {
        title: config.leaderboard.title.clone(),
        sort_order: config.leaderboard.sort_order,
    })
}

// ---------------------------------------------------------------------------
// OpenAPI document
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Luddy Hackathon SP26 API",
        version = "1.0.0",
        description = "REST API for the Luddy Hackathon SP26 leaderboard service.
Participants submit scores via `/add` and the server maintains a live ranked \
leaderboard.  Administrative operations (e.g. updating the board title or sort \
order) require a Bearer token in the `Authorization` header."
    ),
    paths(
        // Health
        health_handler,
        // Scores
        add_handler,
        remove_handler,
        info_handler,
        routes::history::get_history,
        routes::history::user_history,
        // Leaderboard
        routes::leaderboard::get_leaderboard,
        routes::leaderboard::get_leaderboard_json,
        routes::leaderboard::get_leaderboard_num,
        routes::leaderboard::get_leaderboard_json_num,
        // Admin
        routes::admin::update_config_handler,
        // Config / Metrics
        board_config_handler,
        performance_handler,
    ),
    components(
        schemas(
            HealthResponse,
            AddRequest,
            PerformanceEntry,
            BoardConfigResponse,
            Score,
            ScoreStats,
            ScoreHistory,
            LeaderboardSortOrder,
            Config,
        )
    ),
    tags(
        (name = "health",  description = "Service liveness checks"),
        (name = "scores",  description = "Score submission, removal, and statistics"),
        (name = "leaderboard", description = "Ranked leaderboard views (plain-text and JSON)"),
        (name = "admin",   description = "Protected administrative operations (Bearer token required)"),
        (name = "config",  description = "Read the current leaderboard configuration"),
        (name = "metrics", description = "Internal request-latency metrics"),
    )
)]
pub struct ApiDoc;

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

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
        .route("/boardconfig", get(board_config_handler))
        .nest("/history", routes::history::router())
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
