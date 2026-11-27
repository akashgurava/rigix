use std::net::SocketAddr;

use axum::http::HeaderName;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::router::app;

pub(crate) static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

tokio::task_local! {
    pub(crate) static REQUEST_ID: String;
}

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

pub async fn run(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app()).await?;

    Ok(())
}
