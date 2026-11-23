pub mod db;
pub mod error;
pub mod hx;
pub mod routes;
pub mod views;

use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use sqlx::SqlitePool;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct App {
    pub pool: SqlitePool,
}

pub fn build_app(pool: SqlitePool) -> Router {
    let state = App { pool };
    Router::new()
        .route("/", get(routes::items::index))
        .route(
            "/items",
            get(routes::items::index).post(routes::items::create),
        )
        .route("/items/{id}", axum::routing::delete(routes::items::delete))
        .route("/static/{*path}", get(routes::assets::serve))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(TraceLayer::new_for_http())
}
