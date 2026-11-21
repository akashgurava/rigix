# Rigix Architecture & Implementation Guide

## Vision
Rigix is an opinionated, compile-time-enforced Rust application framework and metaprogramming DSL centered on a pure user event flow (where requests convert to responses). It eliminates multi-crate boilerplate and manual policing of standards across our projects (Kurator and 4-5 upcoming projects).

## Core Deliverables
1. **Workspace & Tooling**:
   - Cargo workspace with `crates/rigix-macros` and `crates/rigix`.
   - `dev.sh` workspace orchestrator (`doctor`, `lint`, `fmt`, `build`, `test`, `audit`).
2. **`crates/rigix-macros` (`proc-macro = true`)**:
   - `#[derive(Request)]`: Emits encapsulated inbound request/command DTOs (private fields enforced at compile time, reference/Copy getters, `into_parts`).
   - `#[derive(Response)]`: Emits encapsulated outbound response DTOs (private fields enforced at compile time, reference/Copy getters, `into_parts`).
   - `#[route]`: Emits OpenAPI metadata (`utoipa::path`), `ApiEnvelope` response formatting, and route registration.
3. **`crates/rigix` (Facade Crate)**:
   - Feature flags: `derive` (default), `server`, `full` (`derive` + `server`).
   - Built-in Axum runtime host (`AppHost`), CLI flags (`--host`, `--port`, `--ui-dir`), `/api/v1/health`, and Swagger UI mounting.
4. **Integration Tests**:
   - End-to-end tests verifying the request-to-response user event flow and `#[route]` in a sample app.
