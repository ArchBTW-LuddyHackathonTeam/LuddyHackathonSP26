use axum::{
    Router,
    body::Body,
    http::header::{self},
    response::Response,
    routing::get,
};

use crate::router::AppState;

async fn demo_stylesheet() -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(Body::from(include_str!("../../demo/style.css")))
        .unwrap()
}

async fn demo_script() -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/javascript")
        .body(Body::from(include_str!("../../demo/script.js")))
        .unwrap()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/style.css", get(demo_stylesheet))
        .route("/script.js", get(demo_script))
}
