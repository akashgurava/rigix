# Research: DuckDB and SQLite Pluggable Connection & Driver Constraints

**Ticket Reference:** RGX-7 (nbifmyelu1wkugikvrry597x)  
**Status:** Completed  
**Topic:** Dynamic Runtime Driver Switching (SQLite, DuckDB, PostgreSQL), Container Runtime Linking (`gcr.io/distroless/cc-debian13`), Concurrency/Locking Semantics, and Build Pipeline Constraints.

---

## 1. Executive Summary

Rigix requires a pluggable storage layer capable of dynamically switching at runtime between:
1. **Embedded SQLite** (synchronous C engine, optimized for lightweight, single-node transactional OLTP metadata),
2. **Embedded DuckDB** (synchronous C++ engine, optimized for in-process columnar analytics/OLAP and vector queries), and
3. **Remote PostgreSQL** (asynchronous network client, optimized for multi-client, distributed, highly concurrent workloads).

This switching must occur **without ORM macros** (e.g., Diesel or SeaORM) via connection string parsing, while compiling into a secure, minimal container image based on `gcr.io/distroless/cc-debian13`.

### Key Findings & Verdict
* **Dynamic Driver Switching:** Attempting to build a "universal dynamic SQL generator" without an ORM fails in production due to severe SQL dialect divergence (parameter placeholders, types, locking hints, and upsert syntax). The proven, high-performance architectural pattern in Rust is the **Domain Repository Pattern** (`Arc<dyn DomainRepository>`) backed by driver-specific adapters. This bridges asynchronous network I/O (`sqlx::PgPool`) and synchronous embedded C/C++ engines (`rusqlite` / `duckdb`) via Tokio's blocking thread pool (`tokio::task::spawn_blocking`).
* **Container & C++ Linking:** `gcr.io/distroless/cc-debian13` is based on Debian 13 (Trixie) and ships with `glibc` 2.39/2.40, `libstdc++.so.6` (GCC 14), and `libgcc_s.so.1`. Because `libstdc++` is provided dynamically in `cc-debian13`, DuckDB and SQLite compile seamlessly using dynamic C++ runtime linking. However, the Docker build stage **must not** use a Linux distribution with a newer glibc than Debian 13 (e.g. Ubuntu rolling or Debian experimental), or the binary will fail at container startup with `GLIBC_X.XX not found`.
* **Concurrency Models:**
  * **DuckDB:** Single-writer, multi-reader across processes via OS-level exclusive file locks. In-process multi-threading uses MVCC and Optimistic Concurrency Control (OCC). Multiple threads can write concurrently if they touch disjoint rows; row conflicts abort. Vectorized query execution parallelizes across CPU cores internally (`PRAGMA threads=N`).
  * **SQLite:** Requires explicit WAL mode (`PRAGMA journal_mode=WAL;`) and a busy timeout (`PRAGMA busy_timeout=5000;`) to enable multi-reader single-writer concurrency. Write transactions must explicitly use `BEGIN IMMEDIATE` to prevent deadlocks and `SQLITE_BUSY_SNAPSHOT` errors.
  * **PostgreSQL:** Fully distributed, multi-client, row-level locking MVCC over TCP sockets.
* **Build Constraints:** Bundling DuckDB from source via `duckdb-sys` adds **5 to 15+ minutes** to build times, consumes **4–8 GB RAM** during C++ compilation, and adds **40–70 MB** to the stripped executable. SQLite (`rusqlite` bundled) adds only ~15 seconds and ~3 MB. For CI/CD, we must employ `cargo-chef` layer caching, Cargo feature flags, or precompiled library linking (`DUCKDB_DOWNLOAD_LIB=1`).

---

## 2. Dynamic Driver Switching Architecture in Rust

### 2.1 The Fallacy of Universal Dynamic SQL
A common architectural pitfall is attempting to build a generic SQL string builder or relying on generic connection pools (such as `sqlx::AnyPool`). In practice:
1. `sqlx::AnyPool` **does not support DuckDB**. DuckDB is an in-process columnar database with its own C/C++ engine, not an open wire-protocol network database supported by SQLx.
2. The three target engines diverge fundamentally across core SQL features:

| SQL Dimension | Remote PostgreSQL | Embedded SQLite | Embedded DuckDB |
| :--- | :--- | :--- | :--- |
| **Driver Model** | Non-blocking Async (`sqlx::PgPool`) | Synchronous C FFI (`rusqlite`) | Synchronous C++ FFI (`duckdb`) |
| **Parameter Placeholders** | `$1, $2, ...` | `?1, ?2` or `?` | `?` or `$1, $2` |
| **Upsert Syntax** | `ON CONFLICT (col) DO UPDATE ...` | `ON CONFLICT (col) DO UPDATE ...` | `ON CONFLICT (col) DO UPDATE ...` (recent versions) |
| **Data Types** | Strong: `TIMESTAMPTZ`, `UUID`, `JSONB`, `BIGINT` | Weak/Dynamic: `TEXT`, `INTEGER`, `REAL`, `BLOB` | Columnar: `TIMESTAMP WITH TIME ZONE`, `VARCHAR`, `STRUCT`, `LIST` |
| **Identity / Auto-inc** | `GENERATED ALWAYS AS IDENTITY` | `INTEGER PRIMARY KEY AUTOINCREMENT` | `SEQUENCE` / auto-increment column |
| **Returning Clause** | `RETURNING *` | `RETURNING *` (SQLite >= 3.35) | `RETURNING *` |

Attempting to dynamically generate raw SQL strings across these three engines without an ORM introduces immense fragility, subtle dialect bugs, and SQL injection risks.

---

### 2.2 Repository Pattern: Dynamic Trait Objects vs. Enum Dispatch

To achieve clean, robust switching, Rigix must decouple business logic from storage mechanisms using domain-driven **Repository Traits**.

#### Option A: Dynamic Trait Objects (`Arc<dyn Repository>`)
* **Mechanism:** The application state holds trait objects (e.g., `Arc<dyn JobRepository>`). Concrete implementations (`PostgresJobRepository`, `SqliteJobRepository`, `DuckDbJobRepository`) implement the trait.
* **Pros:**
  * Clean architectural separation (Hexagonal / Clean Architecture).
  * Backend implementations can reside in isolated modules or crates without circular dependencies.
  * Trivial mocking in unit tests (`MockJobRepository`).
  * Adding or removing backends does not touch core domain logic.
* **Cons:**
  * Requires dynamic dispatch (vtable lookup). However, dynamic dispatch in Rust takes ~2–5 nanoseconds, which is negligible compared to database I/O (microseconds for in-memory embedded engines, milliseconds for remote PostgreSQL).
  * Requires boxing futures when using async traits (`#[async_trait]` or Rust 1.75+ dyn-compatible traits).

#### Option B: Enum Dispatch (`enum ConnectionPool` / `enum StorageBackend`)
* **Mechanism:** A top-level enum encapsulates the backends:
  ```rust
  pub enum StorageBackend {
      Postgres(sqlx::PgPool),
      Sqlite(Arc<Mutex<rusqlite::Connection>>),
      DuckDb(duckdb::Connection),
  }
  ```
* **Pros:**
  * Static dispatch (zero vtable overhead).
  * Compile-time exhaustive pattern matching.
* **Cons:**
  * Monolithic coupling: Every backend dependency must be linked together.
  * Conditional compilation (`#[cfg(feature = "duckdb")]`) litters match expressions across every database call.
  * Violates the Open/Closed Principle: Adding a storage engine forces modification of the enum and all match statements.

#### Verdict
**Adopt Option A: Dynamic Trait Objects (`Arc<dyn DomainRepository>`)**. The negligible nanosecond overhead of trait object dispatch is completely irrelevant in database operations, while the architectural isolation and testability benefits are immense.

---

### 2.3 Connection String Parsing & Scheme Dispatch

Rigix should parse connection URIs using the standard `url::Url` crate:

```text
postgres://postgres:password@localhost:5432/rigix_db?sslmode=disable
sqlite:///var/data/rigix.db
sqlite::memory:
duckdb:///var/data/rigix.duckdb
duckdb::memory:
```

```rust
use std::sync::Arc;
use url::Url;

pub enum DatabaseConfig {
    Postgres(String),
    Sqlite { path: String, in_memory: bool },
    DuckDb { path: String, in_memory: bool },
}

impl DatabaseConfig {
    pub fn parse(raw_url: &str) -> Result<Self, StorageError> {
        if raw_url == "sqlite::memory:" {
            return Ok(DatabaseConfig::Sqlite { path: ":memory:".into(), in_memory: true });
        }
        if raw_url == "duckdb::memory:" {
            return Ok(DatabaseConfig::DuckDb { path: ":memory:".into(), in_memory: true });
        }

        let parsed = Url::parse(raw_url).map_err(|e| StorageError::InvalidUrl(e.to_string()))?;
        match parsed.scheme() {
            "postgres" | "postgresql" => Ok(DatabaseConfig::Postgres(raw_url.to_string())),
            "sqlite" => {
                let path = parsed.path().to_string();
                Ok(DatabaseConfig::Sqlite { path, in_memory: false })
            }
            "duckdb" => {
                let path = parsed.path().to_string();
                Ok(DatabaseConfig::DuckDb { path, in_memory: false })
            }
            unknown => Err(StorageError::UnsupportedScheme(unknown.to_string())),
        }
    }
}
```

---

### 2.4 Bridging Async Tokio with Synchronous FFI Drivers

* **PostgreSQL (`sqlx`):** Non-blocking async I/O. Awaiting queries yields the Tokio thread back to the runtime executor.
* **SQLite (`rusqlite`) & DuckDB (`duckdb`):** Synchronous C/C++ FFI calls that execute OS system calls (file reads/writes) and CPU-heavy computation in the calling OS thread.

> [!CAUTION]
> Never execute synchronous `rusqlite` or `duckdb` statements directly on a Tokio worker thread. Doing so blocks the Tokio reactor and prevents other tasks from scheduling.

#### Architectural Bridging Patterns
1. **`tokio::task::spawn_blocking` (Standard & Recommended):**
   Offloads the synchronous C/C++ call to Tokio’s dedicated blocking thread pool (`max_blocking_threads`, default 512).
2. **Dedicated Background Actor Thread (Channel-Based):**
   A dedicated OS thread owns the single write connection and receives commands over an `mpsc` channel. Readers use cloned connections or a read pool via `spawn_blocking`.

#### Production Code Blueprint

```rust
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub status: String,
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Job>, StorageError>;
    async fn insert(&self, job: &Job) -> Result<(), StorageError>;
}

// -------------------------------------------------------------
// PostgreSQL Implementation (Async Native)
// -------------------------------------------------------------
pub struct PostgresJobRepository {
    pool: sqlx::PgPool,
}

#[async_trait]
impl JobRepository for PostgresJobRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Job>, StorageError> {
        let row = sqlx::query_as!(
            JobRecord,
            "SELECT id, status FROM jobs WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(row.map(|r| Job { id: r.id, status: r.status }))
    }

    async fn insert(&self, job: &Job) -> Result<(), StorageError> {
        sqlx::query!(
            "INSERT INTO jobs (id, status) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET status = $2",
            job.id,
            job.status
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(e.to_string()))?;
        Ok(())
    }
}

// -------------------------------------------------------------
// SQLite Implementation (Synchronous FFI + spawn_blocking)
// -------------------------------------------------------------
pub struct SqliteJobRepository {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

#[async_trait]
impl JobRepository for SqliteJobRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Job>, StorageError> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT id, status FROM jobs WHERE id = ?1")
                .map_err(|e| StorageError::Query(e.to_string()))?;
            
            let mut rows = stmt.query_map([id], |row| {
                Ok(Job { id: row.get(0)?, status: row.get(1)? })
            }).map_err(|e| StorageError::Query(e.to_string()))?;

            match rows.next() {
                Some(Ok(job)) => Ok(Some(job)),
                Some(Err(e)) => Err(StorageError::Query(e.to_string())),
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| StorageError::Internal(e.to_string()))?
    }

    async fn insert(&self, job: &Job) -> Result<(), StorageError> {
        let conn = Arc::clone(&self.conn);
        let job = job.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO jobs (id, status) VALUES (?1, ?2) ON CONFLICT (id) DO UPDATE SET status = ?2",
                rusqlite::params![job.id, job.status],
            ).map_err(|e| StorageError::Query(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::Internal(e.to_string()))?
    }
}

// -------------------------------------------------------------
// DuckDB Implementation (Synchronous FFI + spawn_blocking)
// -------------------------------------------------------------
pub struct DuckDbJobRepository {
    conn: Arc<Mutex<duckdb::Connection>>,
}

#[async_trait]
impl JobRepository for DuckDbJobRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Job>, StorageError> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT id, status FROM jobs WHERE id = ?")
                .map_err(|e| StorageError::Query(e.to_string()))?;
            
            let mut rows = stmt.query_map([id], |row| {
                Ok(Job { id: row.get(0)?, status: row.get(1)? })
            }).map_err(|e| StorageError::Query(e.to_string()))?;

            match rows.next() {
                Some(Ok(job)) => Ok(Some(job)),
                Some(Err(e)) => Err(StorageError::Query(e.to_string())),
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| StorageError::Internal(e.to_string()))?
    }

    async fn insert(&self, job: &Job) -> Result<(), StorageError> {
        let conn = Arc::clone(&self.conn);
        let job = job.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO jobs (id, status) VALUES (?, ?)",
                duckdb::params![job.id, job.status],
            ).map_err(|e| StorageError::Query(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::Internal(e.to_string()))?
    }
}
```

---

## 3. Container & Runtime Linking: `gcr.io/distroless/cc-debian13`

When running Rust services with embedded C/C++ dependencies in containerized environments, binary linking and OS ABI compatibility require strict adherence to Debian version boundaries.

### 3.1 What is `gcr.io/distroless/cc-debian13`?
Google's distroless images are stripped-down execution environments without shells, package managers, or standard Linux userland binaries (`bash`, `sh`, `apt`, `coreutils`).

The **`cc-debian13`** image is based on **Debian 13 (Trixie)**:
* **Included Libraries:**
  * `glibc` (`libc.so.6`, `libm.so.6`, `libpthread.so.0`)
  * Dynamic loader: `/lib64/ld-linux-x86-64.so.2`
  * GNU C++ runtime: `libstdc++.so.6` (GCC 14 / `GLIBCXX_3.4.33`)
  * GCC runtime: `libgcc_s.so.1`
  * System CA certificates (`/etc/ssl/certs/ca-certificates.crt`) and timezone data.
* **Difference from `base-debian13` and `static-debian13`:**
  * `distroless/static-debian13`: Contains NO glibc and NO libstdc++. Only statically linked musl binaries can run here.
  * `distroless/base-debian13`: Contains glibc, but **lacks `libstdc++.so.6`**. A C++ application (DuckDB) dynamically linked against libstdc++ will fail immediately with:
    `error while loading shared libraries: libstdc++.so.6: cannot open shared object file: No such file or directory`.
  * `distroless/cc-debian13`: Specifically provides `libstdc++.so.6` and `libgcc_s.so.1`, making it the exact target for DuckDB and SQLite.

---

### 3.2 The Glibc Compatibility Rule (Builder vs. Target)

> [!IMPORTANT]
> In Linux dynamic linking, binaries are backward compatible with older glibc builds, but **not forward compatible**. A binary compiled against `GLIBC_2.40` cannot run on a runtime with `GLIBC_2.36`.

* Debian 13 (Trixie) uses `glibc` versions **2.39 to 2.41**.
* If the multi-stage Docker builder uses Debian 12 Bookworm (`rust:1.x-bookworm`, glibc 2.36), the compiled binary references symbols up to `GLIBC_2.36`. It will run smoothly on `cc-debian13` (glibc 2.39/2.40).
* However, if the builder stage uses an experimental OS with a newer glibc than Debian 13 (e.g. bleeding-edge Ubuntu or Debian experimental), the binary will fail on `cc-debian13` with:
  ```text
  /lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_X.XX' not found (required by ./rigix-server)
  ```
* **Best Practice:** Use `rust:1.xx-bookworm` or `rust:1.xx-trixie` (or `debian:trixie-slim`) as the builder base image.

---

### 3.3 Dynamic vs. Static C++ Linking

When compiling `duckdb-sys` and `rusqlite`:
* **SQLite:** Written in C99. Links against standard C runtime (`libc`).
* **DuckDB:** Written in modern C++17/C++20. Links against `libstdc++`.

#### Dynamic Linking (Recommended for `cc-debian13`)
* By default, the `cc` crate links C++ libraries dynamically via `-lstdc++`.
* Because `cc-debian13` includes `/usr/lib/x86_64-linux-gnu/libstdc++.so.6`, dynamic linking works out of the box.
* **Benefits:**
  * Smaller binary size (avoids embedding 2–4 MB of static libstdc++ into the binary).
  * Standard GNU ABI compatibility with the container's base system.

#### Static Linking (`-static-libstdc++`)
* Can be forced via Cargo rustc flags:
  ```toml
  # In .cargo/config.toml
  [target.x86_64-unknown-linux-gnu]
  rustflags = ["-C", "link-arg=-static-libstdc++", "-C", "link-arg=-static-libgcc"]
  ```
* While this allows running on `distroless/base-debian13`, it is unnecessary on `distroless/cc-debian13` and can occasionally cause static initialization ordering issues with C++ exception handling.

---

### 3.4 Multi-Stage Dockerfile Blueprint

Here is the production-ready multi-stage Dockerfile tailored for Rigix:

```dockerfile
# -------------------------------------------------------------
# Stage 1: Dependency Chef (Build Caching)
# -------------------------------------------------------------
FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    g++ \
    cmake \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# -------------------------------------------------------------
# Stage 2: Recipe Planner
# -------------------------------------------------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# -------------------------------------------------------------
# Stage 3: Builder
# -------------------------------------------------------------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build and cache dependencies (including the heavy DuckDB C++ compilation)
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build the final binary
COPY . .
RUN cargo build --release --bin rigix-server

# Strip debug symbols to minimize final binary size
RUN strip target/release/rigix-server

# -------------------------------------------------------------
# Stage 4: Production Distroless CC Runtime
# -------------------------------------------------------------
FROM gcr.io/distroless/cc-debian13:nonroot

WORKDIR /app

# Copy the stripped Rust binary
COPY --from=builder /app/target/release/rigix-server /app/rigix-server

# Copy SSL certs (already in distroless, but ensured here)
USER nonroot:nonroot

ENTRYPOINT ["/app/rigix-server"]
```

---

## 4. Concurrency & Locking Semantics

A critical dimension when switching between SQLite, DuckDB, and PostgreSQL is their drastically different concurrency guarantees.

### 4.1 DuckDB Concurrency Model
DuckDB is an **embedded, in-process columnar database** designed for OLAP (analytical queries, aggregations, vector processing).

#### Multi-Process Constraints
* **Single Writer Lock:** When a process opens a DuckDB file in read-write mode, DuckDB acquires an **exclusive OS file lock**.
* If a second operating system process attempts to open the same file for writing, it immediately fails:
  ```text
  IO Error: Could not set lock on file "data.duckdb": Resource temporarily unavailable
  ```
* **Multi-Process Reads:** Multiple processes can concurrently open the same DuckDB file **only if all processes connect with `access_mode = 'READ_ONLY'`**. A read-only process cannot access the file while a read-write process holds the write lock.

#### In-Process Multi-Threading
* **MVCC & OCC:** Within a single process, multiple threads can access DuckDB concurrently.
* **Concurrent Reads:** Reader threads never block other reader threads and see a consistent point-in-time snapshot of the database.
* **Concurrent Writes:** DuckDB allows multiple threads within the same process to execute write transactions concurrently using **Optimistic Concurrency Control (OCC)**. Appends to tables do not conflict. However, if two concurrent threads attempt to update or delete the exact same row, OCC triggers a serialization failure: the second transaction is aborted with a transaction conflict error.
* **Connection Thread Safety:** A `duckdb::Connection` is **`!Sync`** (it contains internal mutable C state). To share access across threads:
  1. Use `conn.try_clone()?` to create independent connection handles referencing the same underlying database instance, or
  2. Protect the connection with `Arc<Mutex<duckdb::Connection>>`, or
  3. Use an async actor queue.
* **Internal Query Parallelism:** When executing an analytical query, DuckDB automatically parallelizes execution across multiple CPU cores via internal worker threads (`PRAGMA threads=N`).

---

### 4.2 SQLite Concurrency Model
SQLite is an **embedded, in-process row-oriented database** designed for OLTP (record lookups, transactional state).

#### Rollback Journal (Default) vs. WAL Mode
* Under the default rollback journal mode, writing locks the entire database file. Readers block writers, and writers block readers.
* **Write-Ahead Logging (WAL):** When `PRAGMA journal_mode = WAL;` is enabled:
  * Readers do not block writers.
  * Writers do not block readers.
  * Readers query the main database file and read recent modifications from the `.db-wal` file.
  * **Single Writer Rule:** SQLite permits **strictly one writer at any instant**. Multiple concurrent writes are queued.

#### Critical SQLite Configuration for Multi-Threading
To avoid `SQLITE_BUSY` ("database is locked") errors in a multi-threaded Rust server:
1. **Busy Timeout:** Always configure `PRAGMA busy_timeout = 5000;`. If another connection is writing, SQLite sleeps and retries for up to 5,000 ms before returning an error.
2. **`BEGIN IMMEDIATE` for Write Transactions:**
   Standard transactions begin with `BEGIN DEFERRED`. If Thread A and Thread B both begin deferred transactions, read some data, and then both execute `UPDATE`, SQLite attempts to promote both read locks to reserved write locks. This causes an unavoidable deadlock, resulting in `SQLITE_BUSY_SNAPSHOT`.
   * **Solution:** Any transaction that will write must start with `conn.execute_batch("BEGIN IMMEDIATE")`. This acquires the write lock upfront, preventing race conditions.

---

### 4.3 Concurrency & Workload Comparison Matrix

| Property | Embedded SQLite (WAL) | Embedded DuckDB | Remote PostgreSQL |
| :--- | :--- | :--- | :--- |
| **Primary Workload** | Transactional (OLTP) | Columnar / Analytics (OLAP) | Enterprise OLTP & OLAP |
| **Max Concurrent Writers** | 1 (serialized across system) | 1 per file (multi-thread OCC) | Thousands (row-level locking) |
| **Max Concurrent Readers** | Many (non-blocking in WAL) | Many (non-blocking MVCC) | Thousands (MVCC) |
| **Multi-Process Access** | Yes (shared WAL & locks) | No (single process for RW) | Yes (client-server network) |
| **In-Memory Mode** | `sqlite::memory:` | `duckdb::memory:` | N/A (RAM disk / unlogged) |
| **Internal Parallelism** | Single-threaded per query | Multi-threaded per query | Multi-process/thread per query |
| **Network Overhead** | 0 ms (in-process FFI) | 0 ms (in-process FFI) | 0.5 ms – 5 ms (TCP I/O) |
| **Typical Use in Rigix** | Local metadata & job queues | Metrics, vector search, logs | Production multi-node cluster |

---

## 5. Compile Times, Binary Size, and CI Optimization

Bundling C and C++ amalgamations into a Rust crate has severe implications for compilation performance and container artifact size.

### 5.1 Quantitative Breakdown

| Driver / Dependency | Language | Source Amalgamation Size | Compile Time (Fresh Build) | Peak Memory (RAM) | Stripped Binary Size Contribution |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **`sqlx` (Postgres only)** | Pure Rust | ~50 KLOC Rust | ~25–40 sec | ~1.2 GB | ~8–12 MB |
| **`rusqlite` (`bundled`)** | C99 | ~250 KLOC C (`sqlite3.c`) | ~15–25 sec | ~0.8 GB | ~2–4 MB |
| **`duckdb` (`bundled`)** | C++17/20 | > 1.2 MLOC C++ (`duckdb.cpp`) | **5–15+ min** | **4–8 GB** | **40–70 MB** |
| **Combined (All 3)** | Rust + C + C++ | Large Monolith | **7–18 min** | **6–8 GB** | **55–90 MB** |

> [!WARNING]
> Compiling DuckDB with `features = ["bundled"]` compiles a massive C++ codebase. In CI environments or Docker builds with high concurrency (`-j$(nproc)` on 4–8 vCPUs) and limited memory (e.g., standard GitHub Actions with 7 GB RAM), compilation will frequently trigger the Linux Out-Of-Memory (OOM) killer.

---

### 5.2 Build Mitigation Strategies

To maintain fast developer feedback loops and rapid CI pipelines:

#### Strategy 1: Cargo Feature Gating (Modular Compilation)
Do not force every developer or CI build to compile DuckDB if they only test PostgreSQL or SQLite.
```toml
[features]
default = ["sqlite", "postgres"]
sqlite = ["dep:rusqlite"]
postgres = ["dep:sqlx"]
duckdb-backend = ["dep:duckdb"]

[dependencies]
rusqlite = { version = "0.33", features = ["bundled"], optional = true }
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres"], optional = true }
duckdb = { version = "1.1", features = ["bundled"], optional = true }
```

#### Strategy 2: Precompiled DuckDB Dynamic Library (`DUCKDB_DOWNLOAD_LIB=1`)
Instead of compiling DuckDB's 1.2 million lines of C++ on every clean build, `libduckdb-sys` supports downloading precompiled dynamic libraries directly:
```bash
export DUCKDB_DOWNLOAD_LIB=1
cargo build --release
```
This reduces the DuckDB compilation step from **10 minutes down to ~5 seconds** (downloading the prebuilt `.so` library), while linking dynamically against `cc-debian13`.

#### Strategy 3: Docker Layer Caching with `cargo-chef`
Use `cargo-chef` in the Dockerfile (as demonstrated in Section 3.4). Once the DuckDB and SQLite C/C++ dependencies are compiled and cached in the Docker layer, application code changes recompile in under 10 seconds.

#### Strategy 4: Binary Size Reduction via Release Profile
Add size-optimization flags to `Cargo.toml`:
```toml
[profile.release]
opt-level = 3          # Max performance for database queries
strip = true           # Automatically strip symbols
lto = "thin"           # Link-Time Optimization across C++ and Rust
codegen-units = 1      # Improves dead-code elimination across boundaries
panic = "abort"        # Eliminates unwinding landing pads (~2-4 MB savings)
```

---

## 6. Recommendations for Rigix

1. **Architecture:**
   * Adopt domain-driven **Repository Traits** (`Arc<dyn JobRepository>`, `Arc<dyn MetricsRepository>`) rather than trying to build a generic SQL generator.
   * Parse database connection URIs (`postgres://`, `sqlite://`, `duckdb://`) in a central storage factory to instantiate the appropriate repository implementation.
   * Encapsulate `rusqlite` and `duckdb` synchronous operations inside `tokio::task::spawn_blocking` to prevent starving the Tokio worker threads.
2. **Container Image:**
   * Standardize on **`gcr.io/distroless/cc-debian13:nonroot`** as the production runtime image.
   * Use **Debian Bookworm or Trixie** in the builder stage to ensure strict glibc backward compatibility.
   * Rely on dynamic linking for `libstdc++.so.6` in `cc-debian13`.
3. **Embedded Database Tuning:**
   * **SQLite:** Enforce `PRAGMA journal_mode = WAL;`, `PRAGMA busy_timeout = 5000;`, and `BEGIN IMMEDIATE` on all write transactions.
   * **DuckDB:** Restrict write access to a single internal actor or Mutex-wrapped connection; clone connections via `try_clone()` for reader tasks.
4. **Build Optimization:**
   * Feature-gate `duckdb-backend` in Cargo so that core development and unit test iterations remain instantaneous (< 30 seconds).
   * Integrate `cargo-chef` into Docker build pipelines to cache C++ compilation artifacts.

---
*Document authored for Rigix Project Specification RGX-7.*
