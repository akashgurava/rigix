# Client Local Storage & Outbox Specification

## Problem Statement

Mobile, desktop, and web users using multi-platform applications experience network drops and intermittent connectivity. Without reliable local persistence, users face blank loading screens or lost actions. However, traditional client-side offline architectures introduce complex caching interceptors, optimistic state drift, and brittle ORM entity-shredding that diverge from backend API contracts and violate the server's Single Source of Truth (SSOT).

## Solution

Rigix implements a decoupled, presentation-only local persistence architecture. Clients maintain an offline response fallback cache (`read_cache`) that stores unparsed raw OpenAPI JSON envelopes to guarantee graceful offline fallback without ORM shredding, paired with a persistent sequential FIFO Mutation Queue (`mutation_outbox`) with UUID idempotency keys that dispatches queued mutations in strict order upon reconnection without optimistic UI drift.

## User Stories

1. As a mobile or desktop user, I want previously viewed screens to load instantly from local cache when disconnected, so that I never see blank screens during network outages.
2. As an offline user, I want my actions (creations, edits, deletions) saved locally in an outbox queue, so that my actions are never lost when I lose internet connectivity.
3. As an application user, I want queued offline mutations dispatched sequentially in FIFO order when reconnected, so that dependent operations execute in the exact order I performed them.
4. As an application developer, I want local storage to store raw OpenAPI JSON envelopes rather than shredded relational tables, so that backend API updates require zero client database migrations.
5. As an application developer, I want identical outbox and cache schemas across Android, Apple, Flutter, Web, and Tauri, so that client platforms share identical behavioral contracts.
6. As an application user, I want retryable network errors (e.g. 500 server errors or connection dropouts) retried with exponential backoff and jitter, so that temporary server hiccups resolve automatically without user intervention.
7. As an application user, I want 4xx client validation errors flagged as fatal and removed from the active queue, so that invalid requests do not permanently block subsequent valid mutations.
8. As a security-conscious user, I want sensitive JWT tokens kept in platform-native secure storage (Keychain, Keystore, SecureStorage), so that credentials are protected by OS-level hardware security.
9. As an application developer, I want the client UI to remain transparent and fetch directly from the server when online, so that the server remains the unquestioned Single Source of Truth.
10. As a multi-platform operator, I want each platform to use its standard lightweight storage engine (Jetpack Room, GRDB.swift, Drift, IndexedDB), so that storage integrates idiomatically with each operating system.
11. As an application developer, I want mutation requests to include client-generated UUID idempotency keys, so that retried requests are never processed twice by the server.
12. As a web and desktop user, I want local storage implemented via IndexedDB without main-thread blocking, so that caching operations do not cause UI frame drops.

## Implementation Decisions

- **Storage Engines per Target Platform**: Standardized on idiomatic, lightweight storage engines across all client targets:
  - Android: Android Jetpack Room with SQLite (`androidx.room`).
  - Apple (iOS/macOS): GRDB.swift with SQLite (WAL mode).
  - Flutter: Drift with SQLite (`drift` / `drift_flutter`).
  - Web CSR & Tauri Desktop: IndexedDB (via `idb`), ensuring an identical static bundle without IPC storage divergence.
- **Raw OpenAPI Envelope Model**: Storage engines store unparsed JSON payloads keyed directly by endpoint URL, delegating 100% of typed serialization and deserialization to official OpenAPI code generators (`openapi-typescript`, `swift-openapi-generator`, `openapi-generator-cli`).
- **Standardized Schemas**:
  - `read_cache`: Stores the latest server response per normalized endpoint URL with a timestamp for offline fallback.
  - `mutation_outbox`: Stores sequential user actions with monotonically increasing auto-increment ID, client-generated UUID idempotency key, target endpoint, HTTP method, raw JSON payload, creation timestamp, retry count, and status.
- **Mutation Status State Machine**:
  - `PENDING`: Queued locally and awaiting sequential dispatch.
  - `IN_FLIGHT`: Currently in transit across the network; prevents duplicate parallel dispatches.
  - `FAILED_RETRYABLE`: Dispatch failed due to network outage or 5xx server error; retried using exponential backoff with jitter.
  - `FAILED_FATAL`: Dispatch failed with 4xx client/validation error; halts head blocking, alerts the user, and logs the failure.
- **Transparent Online Execution**: When online, client UI fetches directly from the server without intermediate caching interceptors. Successful responses mirror to `read_cache` strictly as an offline fallback.
- **Security Sandboxing**: Sensitive JWT tokens are kept strictly in platform-native secure storage (Keychain, Keystore, SecureStorage). General offline data lives in standard OS-sandboxed application storage without SQLCipher overhead.

## Testing Decisions

- **Good Test Definition**: Good client storage tests test the external queue and cache behavior under simulated online and offline network states, validating sequential FIFO ordering, idempotency headers, backoff retries, and fatal status transitions without testing underlying SQLite or IndexedDB driver internals.
- **Modules Tested**:
  - `read_cache` store: verifies write-on-success and offline fallback retrieval.
  - `mutation_outbox` queue: verifies strict FIFO sequence execution and UUID idempotency generation.
  - Retry dispatcher: verifies exponential backoff retry on 5xx/network drop and reset to `PENDING`.
  - Fatal error handler: verifies 4xx status transition to `FAILED_FATAL` without blocking the queue indefinitely.
- **Prior Art**: Unit tests utilizing mock HTTP client handlers simulating intermittent connectivity and status responses.

## Out of Scope

- Optimistic UI updates while offline (UI state strictly mirrors server truth).
- Client-side distributed conflict resolution or CRDT syncing.
- Full-database encryption for non-sensitive cached analytical records.
- Stale-While-Revalidate (SWR) revalidation interceptors (deferred to future offline optimization milestones).

## Further Notes

- Implements the architectural decisions recorded in `docs/adr/0003-client-local-storage-and-outbox.md`.
