use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use crate::app::REQUEST_ID;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("System time error: {source}")]
    SystemTimeError {
        #[source]
        source: std::time::SystemTimeError,
        origin: &'static str,
    },
    #[error("Timestamp overflow: {source}")]
    TimestampOverflow {
        #[source]
        source: std::num::TryFromIntError,
        origin: &'static str,
    },
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            AppError::SystemTimeError { .. } => "SYSTEM_TIME_ERROR",
            AppError::TimestampOverflow { .. } => "TIMESTAMP_OVERFLOW",
        }
    }

    fn origin(&self) -> &'static str {
        match self {
            AppError::SystemTimeError { origin, .. }
            | AppError::TimestampOverflow { origin, .. } => origin,
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::SystemTimeError { .. } | AppError::TimestampOverflow { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn public_message(&self) -> (&'static str, Option<&'static str>) {
        match self {
            AppError::SystemTimeError { .. } | AppError::TimestampOverflow { .. } => (
                "Internal Server Error",
                Some("Could not determine server time"),
            ),
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorMessage {
    short: String,
    extra: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: ErrorMessage,
    request_id: Option<String>,
    origin: Option<&'static str>,
    repair_guide: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code();
        let origin = self.origin();
        let request_id = REQUEST_ID.try_with(|id| id.clone()).ok();

        if status.is_server_error() {
            tracing::error!(
                error_code = code,
                origin = origin,
                request_id = request_id.as_deref().unwrap_or("unknown"),
                error = %self,
                "Server error"
            );
        } else {
            tracing::warn!(
                error_code = code,
                origin = origin,
                request_id = request_id.as_deref().unwrap_or("unknown"),
                error = %self,
                "Client error"
            );
        }

        let (short, extra) = self.public_message();
        let message = ErrorMessage {
            short: short.to_string(),
            extra: if cfg!(debug_assertions) {
                Some(self.to_string())
            } else {
                extra.map(str::to_string)
            },
        };

        let (origin_field, repair_guide) = if cfg!(debug_assertions) {
            (Some(origin), Some(format!("docs/errors.md#{code}")))
        } else {
            (None, None)
        };

        let body = ErrorBody {
            code,
            message,
            request_id,
            origin: origin_field,
            repair_guide,
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[tokio::test]
    async fn test_app_error_into_response() {
        let past = SystemTime::now() - Duration::from_secs(60);
        let now = SystemTime::now();
        let time_err = past.duration_since(now).unwrap_err();

        let app_err = AppError::SystemTimeError {
            source: time_err,
            origin: "test_origin",
        };

        let response = app_err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();

        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body["code"], "SYSTEM_TIME_ERROR");
        assert_eq!(body["origin"], "test_origin");
        assert_eq!(body["repair_guide"], "docs/errors.md#SYSTEM_TIME_ERROR");
        assert_eq!(body["message"]["short"], "Internal Server Error");
        assert!(body["message"]["extra"]
            .as_str()
            .unwrap()
            .contains("System time error"));
    }
}
