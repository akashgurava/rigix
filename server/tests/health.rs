use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let app = server::app();

    let response: Response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));

    let body_bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();

    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body["timestamp"].is_number());
    assert!(body["timestamp"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_health_endpoint_propagates_request_id() {
    let app = server::app();
    let custom_id = "test-req-id-123";

    let response: Response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("x-request-id", custom_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-request-id").unwrap(),
        custom_id
    );
}
