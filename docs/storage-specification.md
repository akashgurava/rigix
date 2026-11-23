# Rigix Storage Specification

**Target Engine:** PostgreSQL 18 (`postgres:18-alpine`)  
**Data Access Stack:** Rust `sqlx` (async `PgPool`, compile-time checked queries, `sqlx::migrate!`)  
**Design Philosophy:** Server-Authoritative SSOT, Zero ORMs, Multi-Tier ELT Pipeline inside PostgreSQL.

---

## 1. Core Principles

1. **PostgreSQL as the Single Source of Truth (SSOT)**: The database is the authoritative owner of data integrity, constraints, triggers, and state transitions. Changing `DATABASE_URL` represents a fresh instance.
2. **Zero ORMs**: Relational database operations, analytical joins, and aggregations are written in native PostgreSQL SQL. We discard ORMs (Diesel, SeaORM) to eliminate impedance mismatch with analytical queries and database triggers.
3. **Compile-Time Safety via SQLx**: All SQL queries executed by the Rust server are verified at compile-time via `sqlx::query!` and `sqlx::query_as!`.
4. **Schema Isolation (Single Connection)**: A single database connection string operates across isolated schemas, keeping template auth separate from downstream business domains.

---

## 2. Infrastructure & Container Environment

### `docker-compose.yml` Blueprint

```yaml
version: "3.9"

services:
  postgres:
    image: postgres:18-alpine
    container_name: rigix-postgres
    restart: unless-stopped
    environment:
      POSTGRES_DB: rigix
      POSTGRES_USER: rigix_user
      POSTGRES_PASSWORD: ${DB_PASSWORD:-rigix_secret}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    command: >
      postgres
        -c shared_buffers=256MB
        -c work_mem=16MB
        -c maintenance_work_mem=64MB
        -c max_connections=100
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rigix_user -d rigix"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### PostgreSQL 18 Features Leveraged
* **Asynchronous I/O (AIO)**: Non-blocking disk reads for sequential table scans and batch aggregations.
* **Index Skip Scans**: Composite B-tree indexes serve queries filtering on trailing columns without redundant single-column index bloat.

---

## 3. Schema Architecture: Multi-Tier Medallion Pipeline

All entities live within the same database instance, partitioned across two dedicated schemas:

```
rigix (database)
├── auth (Schema - Rigix Baseline Infrastructure)
│   ├── users
│   ├── refresh_tokens
│   └── audit_logs
│
└── public (Schema - Downstream Consumer Domain Pipeline)
    ├── Bronze (Raw Ingestion Tables)
    ├── Silver (Cleaned / Deduplicated Relational Tables & Views)
    └── Gold (Analytical Materialized Views for Dashboards)
```

---

## 4. DDL & Baseline Schema (`auth`)

Migrations are managed via standard `.sql` files executed on startup by `sqlx::migrate!("./migrations")`.

### Migration `0001_init_auth.sql`

```sql
-- Create isolated auth schema
CREATE SCHEMA IF NOT EXISTS auth;

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. Users Table
CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON auth.users(email);

-- 2. Refresh Tokens (Rotation & Revocation)
CREATE TABLE IF NOT EXISTS auth.refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    replaced_by_token_id UUID REFERENCES auth.refresh_tokens(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user ON auth.refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_lookup ON auth.refresh_tokens(token_hash) WHERE revoked_at IS NULL;

-- 3. Security Audit Log
CREATE TABLE IF NOT EXISTS auth.audit_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    action VARCHAR(64) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## 5. Downstream Analytical Pipeline (Bronze $\rightarrow$ Silver $\rightarrow$ Gold)

Downstream projects implement dashboard pipelines entirely in PostgreSQL using triggers and materialized views:

### Step 1: Raw Ingestion (Bronze)
Append-only log for incoming mutation payloads and events:
```sql
CREATE TABLE public.raw_events (
    id BIGSERIAL PRIMARY KEY,
    idempotency_key UUID NOT NULL UNIQUE,
    payload JSONB NOT NULL,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Step 2: Deduplication & Transformation (Silver)
Database trigger automatically extracts, types, and dedupes into structured tables:
```sql
CREATE TABLE public.events (
    id UUID PRIMARY KEY,
    metric_name VARCHAR(64) NOT NULL,
    metric_value NUMERIC(12, 4) NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION process_raw_event() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO public.events (id, metric_name, metric_value, occurred_at)
    VALUES (
        NEW.idempotency_key,
        NEW.payload->>'metric_name',
        (NEW.payload->>'metric_value')::NUMERIC,
        (NEW.payload->>'occurred_at')::TIMESTAMPTZ
    )
    ON CONFLICT (id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_process_raw_event
AFTER INSERT ON public.raw_events
FOR EACH ROW EXECUTE FUNCTION process_raw_event();
```

### Step 3: Analytical Materialized View (Gold)
Pre-aggregated dashboard data served directly to frontend clients:
```sql
CREATE MATERIALIZED VIEW public.mv_daily_metrics AS
SELECT
    date_trunc('day', occurred_at) AS metric_date,
    metric_name,
    COUNT(*) AS total_count,
    SUM(metric_value) AS total_volume,
    AVG(metric_value) AS avg_value
FROM public.events
GROUP BY 1, 2
WITH DATA;

-- Unique index required for REFRESH CONCURRENTLY
CREATE UNIQUE INDEX idx_mv_daily_metrics ON public.mv_daily_metrics (metric_date, metric_name);
```

---

## 6. Rust Backend Storage Integration Blueprint

### Connection Pool Initialization (`server/src/db.rs`)

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(25)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await?;

    // Run embedded migrations at startup
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

### Type-Safe Query Execution Example

```rust
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserRecord>, sqlx::Error> {
        sqlx::query_as!(
            UserRecord,
            r#"
            SELECT id, email, is_active, created_at
            FROM auth.users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
    }
}
```

### Asynchronous Materialized View Refresher

```rust
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

pub fn spawn_materialized_view_worker(pool: PgPool, interval: Duration) {
    tokio::spawn(async move {
        loop {
            sleep(interval).await;
            let result = sqlx::query!("REFRESH MATERIALIZED VIEW CONCURRENTLY public.mv_daily_metrics")
                .execute(&pool)
                .await;

            if let Err(e) = result {
                tracing::error!("Failed to refresh materialized view: {e}");
            }
        }
    });
}
```

---

## 7. Storage Deliverables Summary

| Item | Decision |
| :--- | :--- |
| **Engine** | PostgreSQL 18 (`postgres:18-alpine`) |
| **Driver / Pooling** | `sqlx` (v0.8+) with `PgPool` |
| **Query Validation** | Compile-time checked queries (`sqlx::query!`, `sqlx::query_as!`) |
| **ORM** | Definitively excluded |
| **Migrations** | `sqlx::migrate!("./migrations")` executed at startup |
| **Auth Schema** | Dedicated `auth` schema (`users`, `refresh_tokens`, `audit_logs`) |
| **Domain Schema** | `public` dedicated to downstream Medallion pipeline (Bronze $\rightarrow$ Silver $\rightarrow$ Gold) |
| **Analytics Acceleration**| Concurrent Materialized Views (`REFRESH MATERIALIZED VIEW CONCURRENTLY`) via background worker |
