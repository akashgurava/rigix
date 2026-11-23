# Storage & Analytical Pipeline Specification

## Problem Statement

Developers bootstrapping self-hosted, multi-user applications need a robust, authoritative data engine that provides ACID transactional integrity for user authentication and session management while concurrently handling high-volume raw data ingestion and analytical transformations. Existing template solutions either force complex multi-database abstractions, brittle ORM object-relational mappings, or slow on-demand analytical calculations that lock tables and degrade client UI responsiveness.

## Solution

Rigix provides a server-authoritative PostgreSQL 18 storage engine managed via asynchronous connection pooling and compile-time SQL verification (`sqlx`). The database strictly isolates baseline template infrastructure (`auth` schema) from downstream application domains (`public` schema), and establishes a three-tier ELT Medallion pipeline (Bronze raw append log -> Silver deduplicated relational tables -> Gold concurrent analytical materialized views) refreshed asynchronously in the background.

## User Stories

1. As a self-hosted app operator, I want PostgreSQL 18 deployed via Docker with pre-configured healthchecks and memory tuning, so that I have a production-ready, performant database out of the box.
2. As an application developer, I want all SQL queries verified at compile time via `sqlx`, so that typos, type mismatches, and broken queries never reach production.
3. As an application user, I want my authentication credentials and refresh tokens stored with secure rotation and revocation tracking, so that compromised sessions can be invalidated immediately.
4. As an application developer, I want auth tables separated into a dedicated `auth` schema, so that template infrastructure does not pollute or collide with downstream business domain models.
5. As an ingestion pipeline author, I want an append-only raw Bronze table with UUID idempotency keys, so that external webhooks and client mutations can be absorbed without data loss or duplicate processing.
6. As an analytics engineer, I want automatic database triggers to clean and transform raw payloads into Silver relational tables upon insertion, so that operational data is always structured and typed without manual ETL scripts.
7. As a dashboard viewer, I want analytical Gold metrics pre-aggregated into materialized views, so that my dashboard metrics load in sub-millisecond latencies.
8. As an active dashboard user, I want analytical materialized views refreshed concurrently in the background, so that background recalculations never lock read queries or degrade UI responsiveness.
9. As an application operator, I want all schema migrations executed automatically at server startup, so that upgrading the application container applies all database changes without manual intervention.
10. As a security auditor, I want an immutable audit log recording security-critical events (login attempts, token revocations, IP addresses), so that suspicious access patterns can be investigated.
11. As a database administrator, I want composite indexes to support index skip scans, so that queries filtering on trailing columns do not require redundant single-column index bloat.
12. As an application operator, I want standard container healthchecks using `pg_isready`, so that dependent backend services only initialize when the database is fully ready to accept connections.

## Implementation Decisions

- **Engine & Deployment Standard**: Standardized strictly on PostgreSQL 18 (`postgres:18-alpine`) using Docker Compose, leveraging modern PostgreSQL 18 Asynchronous I/O (AIO) and index skip scans.
- **Data Access & Direct SQL**: Standardized on Rust `sqlx` with compile-time query guarantees (`query!`, `query_as!`), async connection pooling (`sqlx::PgPool`), and startup migration execution (`sqlx::migrate!`). Uses PostgreSQL-native DDL, triggers, and views for optimal analytical query performance.
- **Schema Isolation (Single Connection)**: A single connection string operates across isolated schemas:
  - `auth`: Rigix template infrastructure (`users`, `refresh_tokens`, `audit_logs`).
  - `public`: Clean canvas for downstream consumer ingestion tables, triggers, and dashboard views.
- **Medallion Ingestion Architecture**:
  - *Bronze (Raw Layer)*: Append-only table storing incoming mutation payloads and external webhooks with UUID idempotency keys.
  - *Silver (Operational Layer)*: Validated, deduplicated, and typed relational tables populated via PostgreSQL database triggers on Bronze insertions.
  - *Gold (Analytical Layer)*: Denormalized tables and materialized views with unique indexes, optimized for dashboard queries and aggregations.
- **Concurrent Refresh Worker**: Background task in the server runtime executing `REFRESH MATERIALIZED VIEW CONCURRENTLY` at configurable intervals, preventing client read contention during analytical aggregations.
- **Migration Architecture**: Versioned, sequential SQL migration files executed automatically on server startup.

## Testing Decisions

- **Good Test Definition**: Good storage tests test external behavior against a running PostgreSQL instance (via test containers or isolated test databases) through public repository traits. Tests verify end-to-end data integrity, migration idempotency, transaction rollback, and constraint enforcement without mocking SQL drivers or internals.
- **Modules Tested**:
  - Migration runner: verifies clean database bootstrapping and idempotent re-runs.
  - User and Auth repository: verifies password hashing, user registration, and credential verification.
  - Refresh token repository: verifies token rotation, expiration checks, and cascade revocation.
  - Medallion pipeline triggers: verifies Bronze insertions automatically populate typed Silver rows.
  - Materialized view refresh worker: verifies non-blocking concurrent refreshes.
- **Prior Art**: Standard async integration tests using `sqlx::test` with per-test transactional rollback.

## Out of Scope

- Runtime dynamic switching between SQLite, DuckDB, and PostgreSQL (dropped per ADR-0001).
- Client-side storage authority or offline conflict resolution (PostgreSQL is the sole SSOT).
- Third-party authentication integrations (OIDC, LDAP, OAuth2).
- Multi-tenant database partitioning or multi-region replication.

## Further Notes

- Implements the architectural decisions recorded in `docs/adr/0001-postgresql-storage-and-pipeline.md`.
