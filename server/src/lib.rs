use std::time::{SystemTime, UNIX_EPOCH};

use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: u128,
}

pub async fn health_handler() -> Json<HealthResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_millis();

    Json(HealthResponse {
        status: "ok",
        timestamp,
    })
}

pub fn app() -> Router {
    Router::new().route("/api/health", get(health_handler))
}
