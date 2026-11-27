//! HTTP router assembly and request-tracing / correlation middleware.

use axum::{
    body::Body,
    http::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::app::{REQUEST_ID, X_REQUEST_ID};
use crate::handlers::health::health_handler;

/// Binds the incoming `x-request-id` header into task-local storage for downstream error logging.
async fn bind_request_id(request: Request<Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    REQUEST_ID.scope(request_id, next.run(request)).await
}

/// Builds the Axum [`Router`] with all routes and observability middleware layers.
pub fn app() -> Router {
    Router::new()
        .route("/api/health", get(health_handler))
        .layer(middleware::from_fn(bind_request_id))
        .layer(PropagateRequestIdLayer::new(X_REQUEST_ID.clone()))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = request
                    .headers()
                    .get(&X_REQUEST_ID)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown");

                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = %request_id,
                )
            }),
        )
        .layer(SetRequestIdLayer::new(
            X_REQUEST_ID.clone(),
            MakeRequestUuid,
        ))
}
