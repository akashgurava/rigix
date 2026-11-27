//! Server lifecycle, tracing initialization, and request-scoped correlation state.

use std::net::SocketAddr;

use axum::http::HeaderName;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::router::app;

/// HTTP header name used to propagate request correlation IDs.
pub(crate) static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

tokio::task_local! {
    /// Task-local storage holding the active request's correlation ID.
    pub(crate) static REQUEST_ID: String;
}

/// Initializes the global `tracing` subscriber with an environment filter and output format.
pub fn init_tracing(default_filter: &str, log_format: &str) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let registry = tracing_subscriber::registry().with(env_filter);

    if log_format.eq_ignore_ascii_case("json") {
        registry.with(tracing_subscriber::fmt::layer().json()).init();
    } else {
        registry.with(tracing_subscriber::fmt::layer().pretty()).init();
    }
}

/// Binds a TCP listener to `addr` and serves the Axum application.
pub async fn run(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app()).await?;

    Ok(())
}
