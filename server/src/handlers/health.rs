//! Health check endpoint handler.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::error::AppError;
use crate::response::{ServerResponse, ServerSuccess};

/// JSON payload returned by `GET /api/health`.
#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    timestamp: i64,
}

/// Handles `GET /api/health` by returning server status and current epoch milliseconds.
pub(crate) async fn health_handler() -> ServerResponse<HealthResponse> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| AppError::SystemTimeError {
            source,
            origin: "health_handler",
        })?
        .as_millis();

    let timestamp = i64::try_from(millis).map_err(|source| AppError::TimestampOverflow {
        source,
        origin: "health_handler",
    })?;

    Ok(ServerSuccess::ok(HealthResponse {
        status: "ok",
        timestamp,
    }))
}
