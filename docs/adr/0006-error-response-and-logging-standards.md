# ADR-0006: Error, Response, and Logging Standards

## Status
Accepted

## Context
Rigix serves a compiled Rust/Axum backend (`server/`) consumed by multiple decoupled Presentation Shells (`ui/web`, `ui/android`, `ui/apple`, `ui/flutter`, `ui/tauri`) and AI coding assistants.
- Downstream clients rely on HTTP status codes (`4xx` vs `5xx`) for Mutation Queue (FIFO Outbox) Head-of-Line Blocking vs retryable backoff, and need machine-readable error codes rather than parsing English strings.
- AI assistants and developers need coded, searchable error identifiers and call-site origin context to diagnose failures from evidence without digging through obscured stack traces.
- Server-side error logging must happen in a single, deterministic boundary so no error is silently swallowed or logged inconsistently.

## Decision

### 1. Response Structure (`ServerResponse<T>`)
- Handlers return `ServerResponse<T>`, defined as `Result<ServerSuccess<T>, AppError>`.
- **2xx Success (`ServerSuccess<T>`)**: Returns raw JSON `T` without an outer `{ "data": ... }` wrapper (e.g., `200 OK`, `201 Created`, `204 No Content`) for clean `utoipa` OpenAPI codegen and client deserialization.
- **4xx / 5xx Error (`AppError`)**: Returns a structured JSON error object:
  ```json
  {
    "code": "SYSTEM_TIME_ERROR",
    "message": {
      "short": "Internal Server Error",
      "extra": "Could not determine server time"
    },
    "request_id": "019...",
    "origin": "health_handler",
    "repair_guide": "docs/errors.md#SYSTEM_TIME_ERROR"
  }
  ```
  - `code`: Unique, `SCREAMING_SNAKE_CASE`, `git grep`-able identifier.
  - `message`: Human-safe summary. For `5xx` errors, internal details are masked behind `"Internal server error"` in production.
  - `request_id`: Correlates the HTTP response with server logs.
  - `origin` & `repair_guide`: Included in debug/dev builds (and always logged server-side) to point directly to the originating call site and remediation documentation.

### 2. Granular `AppError` & Single-Point Logging
- `AppError` uses specific, granular enum variants (e.g., `DbConnAuthError`, `DbConnInvalidError` rather than generic `DbConnFailedError`).
- `AppError::into_response` is the **single place** on the server where errors are logged (`tracing::error!` for `5xx`, `tracing::warn!` for `4xx`) with full internal cause, `code`, `origin`, and `request_id`.

### 3. Structured Logging & Request Correlation
- `tracing-subscriber` is configured with `EnvFilter` (`RUST_LOG`), defaulting to `pretty` format in debug builds and `json` format in release/production builds (overridable via `LOG_FORMAT=json|pretty`).
- `tower-http` middleware generates or propagates `x-request-id` on every HTTP request, attaches it to the active `tracing` span, and returns it in the `x-request-id` response header.

## Consequences
- **Positive**: Consistent client error handling across all Presentation Shells; zero silent error swallowing; instant `git grep` traceability from error code/origin to source line; AI-ready diagnostics.
- **Negative**: Requires discipline to define specific `AppError` variants for new failure modes rather than collapsing them into generic catch-alls.
