use axum::{
    Router,
    http::{HeaderName, header::CONTENT_TYPE},
    response::AppendHeaders,
    routing::get,
};

use crate::router::AppState;

async fn demo_stylesheet() -> (AppendHeaders<[(HeaderName, &'static str); 1]>, &'static str) {
    (
        AppendHeaders([(CONTENT_TYPE, "text/css")]),
        include_str!("../../demo/style.css"),
    )
}

async fn demo_script() -> (AppendHeaders<[(HeaderName, &'static str); 1]>, &'static str) {
    (
        AppendHeaders([(CONTENT_TYPE, "text/js")]),
        include_str!("../../demo/script.js"),
    )
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/style.css", get(demo_stylesheet))
        .route("/script.js", get(demo_script))
}
