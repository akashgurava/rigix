//! Binary entrypoint for the Rigix server.

use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_filter = "info,tower_http=debug";
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "pretty".to_string()
        } else {
            "json".to_string()
        }
    });

    server::init_tracing(default_filter, &log_format);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8977));
    server::run(addr).await
}
