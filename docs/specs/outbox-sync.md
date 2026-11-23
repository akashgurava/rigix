# Offline Outbox & Cache Synchronization Specification

## Problem Statement

When multi-platform clients operate offline or face intermittent connectivity, buffering user mutations into a persistent queue is essential to prevent data loss. However, replaying queued mutations without a rigorous sync contract risks cascading failures (if an early operation fails and downstream dependent actions execute against invalid server state), excessive network thrashing and UI jitter (if the client triggers re-fetches after every individual drained item), concurrency races across multiple browser tabs or desktop windows, and credential expiration failures from replaying stale tokens.

## Solution

Rigix establishes a strict, server-authoritative offline mutation synchronization and cache revalidation contract. The outbox processor executes sequentially in Head-of-Line (HoL) order, halting immediately on unrecoverable `4xx` client errors to protect causal integrity. Transient network or `5xx` errors are bounded by a 5-attempt exponential backoff curve with full jitter before pausing. Multi-window concurrency is controlled via the native Web Locks API on Web/Tauri and singleton workers on mobile. Query invalidation is debounced until Queue Quiescence, while active background sync is displayed via a dedicated status indicator with a simple dead-letter discard/clear recovery interface.

## User Stories

1. As an application user, I want queued offline mutations processed strictly one at a time in chronological order, so that operations depending on prior actions always execute correctly.
2. As an application user, I want a visible sync indicator while background synchronization is in progress, so that I know pending offline actions are being committed to the server.
3. As an application user, I want the queue to halt immediately if an action fails due to a validation error, so that subsequent actions do not execute against corrupted or missing data.
4. As an application user, I want an error dialog with clear details and options to discard the failing item or clear all pending actions when a sync error halts the queue, so that I can unblock my application cleanly.
5. As an application user, I want temporary network interruptions and 5xx server hiccups retried automatically with jittered exponential backoff up to 5 times, so that brief connection drops resolve seamlessly.
6. As a battery- and data-conscious user, I want automatic retries to pause after 5 consecutive failures during server outages, so that the client does not drain battery or thrash network requests indefinitely.
7. As an application user across multiple browser tabs or desktop windows, I want mutual exclusion over the outbox, so that multiple windows do not concurrently dispatch duplicate mutations or interleave queue orders.
8. As a security-conscious user, I want the outbox to store zero authentication tokens on disk, so that sensitive credentials are never stored unencrypted and re-authentication is verified dynamically upon reconnecting.
9. As an application user, I want the client UI to wait for the outbox to drain before performing a single consolidated data refresh, so that I do not experience visual layout flickering or repeated network re-fetches.
10. As an application developer, I want presentation shells to remain pure presentation without client-side JSON repairing logic, so that client platforms remain completely decoupled from domain validation rules.

## Implementation Decisions

- **Queue Lifecycle & State Transitions**:
  - `PENDING`: Newly buffered mutation awaiting dispatch.
  - `IN_FLIGHT`: Mutation currently in transit over the network.
  - `COMPLETED`: Successfully executed by server (2xx status); removed from active `mutation_outbox`.
  - `FAILED_RETRYABLE`: Transient failure (network disconnect or 5xx status); `retry_count` incremented.
  - `FAILED_FATAL`: Client error (4xx status); triggers Head-of-Line queue halt.
  - `PAUSED`: State entered after 5 consecutive transient failures; requires manual user retry.
- **Head-of-Line (HoL) Blocking**: The outbox worker never skips a failing item. An unrecoverable `FAILED_FATAL` status halts subsequent items at `PENDING` until user intervention.
- **Retry Policy & Jitter Equation**:
  - Exponential backoff with full jitter: `delay = uniform_random(0, min(30000, 1000 * 2^retry_count))` ms.
  - Maximum retry limit: 5 attempts. Exceeding limit transitions queue to `PAUSED`.
- **Quiescence-Driven Cache Sync**:
  - While outbox has items draining, individual screen query re-fetches are suppressed.
  - A global non-blocking sync spinner indicates background processing.
  - Reaching **Queue Quiescence** (queue count = 0 or queue halts in `FAILED_FATAL`/`PAUSED`) triggers a consolidated re-fetch of active screen queries from the backend.
- **Dead-Letter Recovery UI**:
  - On `FAILED_FATAL`, sync indicator turns to error state.
  - Clicking indicator opens modal card displaying HTTP status, endpoint, and server error response body.
  - User options: "Discard Item" (evicts head item, resumes queue) or "Clear All".
- **Multi-Instance Mutual Exclusion**:
  - Web & Tauri: Native Web Locks API (`navigator.locks.request('rigix_outbox_drain')`). Secondary tabs act as observers.
  - Android, Apple, Flutter: Single-instance background worker with in-memory Mutex.
- **Dynamic Auth & Header Binding**:
  - Local database stores strictly `endpoint`, `method`, `payload` (JSON string), `created_at`, `idempotency_key`.
  - Connection manager re-authenticates before queue processing begins.
  - Dispatcher injects `Authorization: Bearer <token>`, `Idempotency-Key: <UUID>`, and `Content-Type: application/json` dynamically.

## Testing Decisions

- **Good Test Definition**: Tests verify queue lifecycle transitions, strict sequential ordering, backoff delays, Web Locks exclusivity, and quiescence cache sync events against mock HTTP server handlers without mocking internal storage drivers.
- **Modules Tested**:
  - Outbox worker: verifies sequential FIFO execution and dynamic header injection.
  - Error classifier: verifies 4xx transitions to `FAILED_FATAL` and halts the queue.
  - Retry scheduler: verifies backoff calculation, full jitter bounds, and 5-attempt `PAUSED` escalation.
  - Quiescence coordinator: verifies re-fetch suppression during draining and single consolidated fetch on quiescence.
  - Web Lock manager: verifies secondary tabs wait and do not execute concurrent dispatches.
- **Prior Art**: Unit and integration test suites using mock HTTP interceptors and synthetic IndexedDB / SQLite test instances.

## Out of Scope

- Optimistic UI state updates prior to server confirmation.
- Client-side conflict resolution or two-way operational transforms (server is SSOT).
- In-place JSON payload editing within the client UI for failed mutations.
- Multi-part or binary file uploads through the outbox queue in v1 (JSON REST only).

## Further Notes

- Implements the architectural decisions recorded in `docs/adr/0004-offline-outbox-and-cache-sync-contract.md`.
- Complements the client storage engine definitions in `docs/adr/0003-client-local-storage-and-outbox.md` and `docs/specs/client-storage.md`.
