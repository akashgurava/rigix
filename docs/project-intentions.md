# Rigix: Project Scope & Intentions

## 1. Core Vision & Goal
- **Self-Hosted Multi-Platform Blueprint**: Rigix is a reusable, domain-agnostic Copier project template built to bootstrap self-hosted, multi-user applications (such as family finance trackers, investment dashboards, or homelab media managers).
- **Server-Authoritative SSOT**: A single compiled Rust executable acts as the Single Source of Truth (SSOT) for data models, persistence, business logic, API contracts (`openapi.json`), and the embedded static web client.
- **Strict Client Decoupling**: Frontend shells contain zero business logic, domain validation, or cross-folder workspace coupling—interacting with the backend strictly via generated OpenAPI contracts and local offline stores.

## 2. Key User & System Capabilities
- **Standardized Server Storage**: Standardized strictly on PostgreSQL 18 with async sqlx connection pooling, compile-time query verification, schema isolation (auth vs public), and concurrent analytical materialized views without ORM impedance mismatch (see ADR-0001).
- **Authentication & Server Health**: Ships out of the box with `GET /health` and DB-backed JWT authentication (signup, login, refresh, logout) featuring refresh-token rotation and server-side revocation.
- **Multi-Platform Client Targets**: Provides decoupled scaffolding for five UI targets (`android`, `apple`, `flutter`, `web`, `tauri`), allowing downstream projects to retain only the platforms they intend to support via Copier prompts.
- **Offline Fallback Cache & FIFO Mutation Queue**: Clients fetch directly from the server when online and render from a local offline response fallback cache (`read_cache`) when disconnected; offline write actions enqueue into a persistent FIFO request queue without optimistic UI updates until server confirmation.
- **Unified Web & Desktop Delivery**: Serves a static client-side rendered (CSR) web bundle directly from the server binary, while Tauri wraps those exact web artifacts natively with a runtime-configurable server URL.

## 3. Tech & Architecture Decisions Agreed On
- **Backend Stack (`server/`)**: Standalone Rust crate using `axum`, `tokio`, `utoipa` (for `openapi.json` generation with typed status responses), `thiserror` for explicit domain errors, and `sqlx` with compile-time query macros (`query!`, `query_as!`), async connection pooling, and explicit domain errors without ORMs.
- **Frontend Architecture (`ui/*`)**: Pure-presentation MVI (Model–View–Intent) shells across Kotlin/Compose (`android`), Swift/SwiftUI (`apple`), Dart (`flutter`), static CSR Solid 2.0 + Tailwind v4 (`ui/web`), and WebView wrapper (`ui/tauri`), all deriving immutable types from `openapi.json`.
- **Client Local Persistence**: Standardized across Jetpack Room (Android), GRDB.swift (Apple), Drift (Flutter), and `idb` IndexedDB (Web/Tauri) with raw OpenAPI JSON envelopes and offline FIFO outbox (see ADR-0003 and `docs/specs/client-storage.md`).
- **Containerization & Healthcheck**: Multi-stage Docker build targeting `gcr.io/distroless/cc-debian13` (providing native glibc and ca-certificates for secure network TLS) with `HEALTHCHECK CMD ["/{{ project_name }}", "healthcheck"]` powered by a synchronous `std::net::TcpStream` subcommand.
- **Template Scaffolding & Docs**: Uses Copier (`{{ project_name }}` substitutions and conditional UI directory generation) with Jinja-commented meta-context headers in `GLOSSARY.md.jinja` and `AGENTS.md.jinja` that strip template-only context in downstream repositories.

## 4. Unresolved Questions & Open Items (The Fog)
- **Hard Lints & Static Analysis**: Finalizing `.clippy.toml` and workspace `[lints]` rules to enforce zero-`unwrap()` and strict encapsulation.
- **Concrete `copier.yml` & Scaffolding Files**: Writing the `copier.yml` prompt definitions alongside the actual starter code for `server/`, the five `ui/*` MVI/cache/outbox skeletons, `Dockerfile`, and `docker-compose.yml`.

## 5. Explicitly Out of Scope for v1
- **Domain-Specific Logic & Feature Views**: Any business calculations, domain schemas, or app-specific screens beyond baseline `/health` and JWT auth belong strictly in downstream projects, not in Rigix.
- **Root Cargo Workspaces & Shared Rust UI Crates**: No root `Cargo.toml` workspace or shared source-level crate coupling between `server/` and `ui/*`.
- **Client Decoupling & Static Delivery**: Clients wait for authoritative server confirmation before mutating state, and `ui/web` delivers static Client-Side Rendered (CSR) assets.
- **Secondary Auth Providers & Monetization**: OIDC/LDAP integrations, external auth providers, and billing/monetization features are excluded from v1.
