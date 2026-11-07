# Rigix Architecture & Implementation Guide

## Vision
Rigix is an opinionated, compile-time-enforced Rust application framework and metaprogramming DSL. It eliminates the multi-crate boilerplate and manual policing of standards across our projects (Kurator and 4-5 upcoming projects).

## Core Deliverables
1. **Workspace & Tooling**:
   - Cargo workspace with `crates/rigix-macros` and `crates/rigix`.
   - `dev.sh` workspace orchestrator (`doctor`, `lint`, `fmt`, `build`, `test`, `audit`).
2. **`crates/rigix-macros` (`proc-macro = true`)**:
   - `#[entity]`: Emits domain model (private fields, reference getters, `try_new`, `into_parts`), wire DTOs (`*Response`, `*Request` with `deny_unknown_fields` and `ToSchema`), and database rows (`*Row` with `FromRow`).
   - `#[route]`: Emits OpenAPI metadata (`utoipa::path`), `ApiEnvelope` response formatting, and route registration.
3. **`crates/rigix` (Facade Crate)**:
   - Feature flags: `macros`, `server`, `auth`, `db`.
   - Built-in Axum runtime host (`AppHost`), CLI flags (`--host`, `--port`, `--ui-dir`, `--db-path`), Argon2 password auth, SQLite session store, `/api/v1/health`, and Swagger UI mounting.
4. **Integration Tests**:
   - End-to-end tests verifying `#[entity]` and `#[route]` in a sample app.
