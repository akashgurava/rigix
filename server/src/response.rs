use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::error::AppError;

pub(crate) enum ServerSuccess<T: Serialize> {
    Ok(T),
}

impl<T: Serialize> ServerSuccess<T> {
    pub(crate) fn ok(data: T) -> Self {
        Self::Ok(data)
    }
}

impl<T: Serialize> IntoResponse for ServerSuccess<T> {
    fn into_response(self) -> Response {
        match self {
            ServerSuccess::Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        }
    }
}

pub(crate) type ServerResponse<T> = Result<ServerSuccess<T>, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_success_into_response() {
        let res = ServerSuccess::ok("hello");
        let response = res.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();

        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body, "hello");
    }
}
