# 0004: Offline FIFO Request Outbox & Cache Sync Contract

## Context
In ADR-0003, Rigix standardized client persistence engines (Room, GRDB, Drift, IndexedDB) and established baseline schemas for the offline response mirror (`read_cache`) and sequential FIFO queue (`mutation_outbox`). However, the exact runtime contract for draining offline mutations, handling unrecoverable client errors versus transient outages, synchronizing UI state without optimistic drift, and managing multi-tab concurrency remained unspecified.

Client offline queue processing faces specific architectural challenges:
1. **Cascading Failure & Queue Poisoning**: Mutations enqueued offline are frequently causally related (e.g. creating a parent resource followed by child updates). Dropping a failed mutation and continuing risks executing subsequent actions against invalid server states.
2. **Cache Synchronization & UI Thrashing**: Triggering individual screen re-fetches for every drained item during a reconnection burst causes high network overhead and visible layout flicker.
3. **Multi-Window Concurrency**: On Web and Tauri desktop, multiple browser tabs or desktop windows opening the same IndexedDB database simultaneously can dispatch requests out-of-order if mutual exclusion is absent.
4. **Credential Expiration**: Storing static HTTP authorization headers in SQLite or IndexedDB leaks bearer tokens into unencrypted storage and risks replaying expired tokens after long offline periods.

## Decision

1. **Head-of-Line (HoL) Blocking & State Transitions**:
   - The FIFO queue processes items strictly in ascending `id` order, one item at a time.
   - **`PENDING` -> `IN_FLIGHT`**: The worker claims the head item and initiates HTTP dispatch.
   - **`IN_FLIGHT` -> `COMPLETED`**: On HTTP 2xx response, the item is deleted from `mutation_outbox` (or moved to historical audit log if configured).
   - **`IN_FLIGHT` -> `FAILED_FATAL`**: On HTTP 4xx client errors (validation, conflict, unauthorized that fails refresh), the queue halts immediately. The head item status transitions to `FAILED_FATAL`, and subsequent queued items remain `PENDING`.
   - **`IN_FLIGHT` -> `FAILED_RETRYABLE`**: On HTTP 5xx errors or network loss, the item transitions to `FAILED_RETRYABLE` with an incremented `retry_count`.

2. **Finite Transient Retry Policy**:
   - Transient failures are retried using truncated exponential backoff with full jitter:
     `delay = random(0, min(max_delay, base_delay * 2^retry_count))`
     where `base_delay = 1000ms`, `max_delay = 30000ms`, and factor = 2.
   - Retries are bounded to a maximum ceiling of **5 attempts** while online.
   - If 5 consecutive attempts fail, the queue transitions to **`PAUSED`** state, stopping automated background attempts and presenting a manual "Retry Sync" action in the UI.

3. **Quiescence-Driven Cache Synchronization & Transparent UI**:
   - While the outbox is actively draining, the UI renders an explicit, non-blocking background sync activity indicator (spinner).
   - Individual screen query re-fetches are suppressed/debounced during draining.
   - Upon reaching **Queue Quiescence** (when the queue reaches 0 pending items or halts in `FAILED_FATAL`/`PAUSED`), active screen queries execute a single consolidated re-fetch from the authoritative backend.
   - The client UI maintains zero optimistic UI drift: state reflects server truth, supplemented only by the sync activity indicator.

4. **Dead-Letter Recovery Workflow**:
   - To preserve the pure-presentation shell architecture, clients do not attempt in-place JSON payload editing for `FAILED_FATAL` mutations.
   - Clicking the sync alert opens a recovery card showing the target endpoint, timestamp, and server error response.
   - The user is offered two actions:
     - **Discard Item**: Removes the faulted head item from `mutation_outbox`, unblocking the queue to resume processing remaining pending mutations.
     - **Clear All**: Clears the entire outbox queue.

5. **Multi-Instance Mutual Exclusion**:
   - **Web & Tauri Desktop**: Outbox queue drainage is guarded by the native **Web Locks API** (`navigator.locks.request('rigix_outbox_drain', ...)`). The lock-holder processes the queue; secondary tabs remain passive observers listening for storage changes.
   - **Android / Apple / Flutter**: Guarded by a singleton worker process backed by an in-memory Mutex / Actor pattern.

6. **Dynamic Authentication Binding**:
   - `mutation_outbox` persists zero authorization headers.
   - Upon network restoration, the platform connection manager verifies/reinstates the active session (refreshing the JWT token via secure storage if needed).
   - The outbox dispatcher injects the fresh `Authorization: Bearer <token>` and `Idempotency-Key: <UUID>` dynamically into the HTTP request envelope at dispatch time.

## Consequences

- Absolute causal integrity: dependent offline operations never execute out-of-order or against corrupted states.
- Clean presentation shells: frontend code remains 100% decoupled from domain schema repairing or relational mapping.
- Minimal battery and network utilization: debounced quiescence re-fetching prevents UI thrashing and excessive server requests.
- Secure credential management: sensitive bearer tokens are never written to disk within the mutation queue.
- User visibility: the background sync indicator ensures users are immediately aware of pending sync and halts.
