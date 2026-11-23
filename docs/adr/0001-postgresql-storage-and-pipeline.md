# 0001: PostgreSQL 18 Authoritative Store & SQLx Pipeline

## Context
Rigix originally considered supporting runtime switching across embedded SQLite, embedded DuckDB, and remote PostgreSQL. However, analysis of downstream OLAP-heavy dashboard workloads with transactional auth revealed that:
1. SQLite's dynamic weak typing and syntax diverge from PostgreSQL, fragmenting migrations and queries.
2. DuckDB's in-process columnar engine with single-writer process locks creates severe friction with frequent single-row OLTP point updates (JWT refresh-token rotation and outbox mutations).
3. Traditional ORMs (Diesel, SeaORM) impose rigid 1:1 table-to-struct mapping and actively fight against multi-tier database pipelines (Raw ingestion $\rightarrow$ Trigger/view deduping $\rightarrow$ Gold analytical materialized views with window functions and complex joins).

## Decision
1. **Engine**: Standardize strictly on **PostgreSQL 18** (`postgres:18-alpine`) as the single authoritative database, eliminating SQLite, DuckDB, and all multi-database abstraction layers.
2. **Query Stack & No ORM**: Standardize on `sqlx` with compile-time query guarantees (`query_as!`), async connection pooling (`sqlx::PgPool`), and migration management (`sqlx::migrate!`), completely rejecting ORMs in favor of PostgreSQL-native DDL, triggers, and views.
3. **Schema Isolation**: Maintain a single database connection string while isolating concerns into PostgreSQL schemas:
   - `auth`: Rigix template plumbing (users, credentials, refresh-token rotation, session revocations).
   - `public`: Clean canvas for downstream consumer ingestion tables, triggers, and dashboard views.
4. **Asynchronous Materialized View Refreshes**: Background workers handle concurrent materialized view refreshes (`REFRESH MATERIALIZED VIEW CONCURRENTLY`) to keep dashboard read latencies in the sub-millisecond range.

## Consequences
- The Rust server executes 100% async-native network I/O via Tokio and SQLx, while PostgreSQL 18 handles non-blocking disk reads via its internal Asynchronous I/O (AIO) engine and optimizes composite queries via index skip scans.
- Developers get compile-time SQL safety without ORM impedance mismatch.
- Template plumbing in `auth` is cleanly decoupled from downstream domain models in `public`.
- Pure standalone execution without a database container is dropped; self-hosting requires running `postgres:18-alpine` (standardized in `docker-compose.yml`).
