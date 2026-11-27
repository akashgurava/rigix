//! Rigix backend server library.
//!
//! Exposes the tracing initializer, HTTP router, and server runner for the binary entrypoint.

mod app;
mod error;
mod handlers;
mod response;
mod router;

pub use app::{init_tracing, run};
pub use router::app;
