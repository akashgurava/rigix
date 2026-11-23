# Rigix Client Local Storage & Outbox Specification

**Target Platforms:** Android, Apple (iOS/macOS), Flutter, Web CSR, Tauri  
**Engines:** Android Jetpack Room (Android), GRDB.swift (Apple), Drift (Flutter), `idb` IndexedDB (Web & Tauri)  
**Design Philosophy:** Server-Authoritative SSOT, Zero Client ORMs, Raw OpenAPI Envelopes, Transparent UI with Offline Fallback.

---

## 1. Core Principles

1. **Server as Single Source of Truth (SSOT)**: When online, client UI targets fetch directly from the server. The UI remains completely transparent and avoids complex client-side caching interceptors.
2. **Offline Read Fallback**: Successful server responses are copied directly into the local `read_cache` table as unparsed JSON envelopes. When offline or unreachable, the UI transparently falls back to this cached payload to prevent blank screens.
3. **FIFO Mutation Queue**: Offline user interactions (creations, edits, deletions) are enqueued into `mutation_outbox` with unique UUID idempotency keys and monotonically increasing IDs, dispatching sequentially upon network restoration without optimistic UI drift.
4. **Zero ORM Entity Shredding**: Local storage avoids shredding API responses into relational domain tables. Payloads are stored as raw JSON strings/blobs, delegating 100% of serialization to official OpenAPI client models.
5. **Storage Security & Sandboxing**: Sensitive JWT tokens are stored exclusively in platform-native secure storage (Keychain, Keystore, SecureStorage). General offline data sits in standard OS-sandboxed application storage without SQLCipher overhead.

---

## 2. Technology Selection per Target

| Target Platform | Storage Engine | Query / Access Model | OpenAPI Generator Toolchain |
| :--- | :--- | :--- | :--- |
| **Android** | **Jetpack Room** (`androidx.room`) | SQLite + Kotlin Coroutines / Flow | `openapi-generator-cli` (`kotlin` + `kotlinx.serialization`) |
| **Apple (iOS/macOS)** | **GRDB.swift** | SQLite (WAL mode) + Swift Concurrency | `swift-openapi-generator` (`Codable` structs) |
| **Flutter** | **Drift** (`drift` / `drift_flutter`) | SQLite + Dart Streams | `openapi-generator-cli` (`dart` + `json_serializable`) |
| **Web CSR** | **`idb`** (IndexedDB) | Structured Object Stores | `openapi-typescript` + `openapi-fetch` |
| **Tauri Desktop** | **`idb`** (IndexedDB via WebView) | Structured Object Stores (Unified bundle) | `openapi-typescript` + `openapi-fetch` |

---

## 3. Storage Schemas

### 3.1 `read_cache` (Offline Response Fallback)

Stores the most recent server response per endpoint for offline presentation fallback.

#### SQLite DDL (Android, Apple, Flutter)
```sql
CREATE TABLE IF NOT EXISTS read_cache (
    endpoint_url TEXT PRIMARY KEY NOT NULL,
    payload TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
```

#### IndexedDB Object Store (Web CSR, Tauri)
- **Store Name:** `read_cache`
- **Key Path:** `endpoint_url` (String)
- **Record Schema:**
  ```ts
  interface ReadCacheEntry {
    endpoint_url: string; // e.g. "/api/v1/metrics?range=24h"
    payload: unknown;      // Raw deserialized JSON or structured object
    updated_at: number;    // Epoch milliseconds
  }
  ```

---

### 3.2 `mutation_outbox` (FIFO Offline Request Queue)

Persistently buffers outgoing user actions while offline to guarantee zero data loss and in-order replay.

#### SQLite DDL (Android, Apple, Flutter)
```sql
CREATE TABLE IF NOT EXISTS mutation_outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'PENDING'
);

CREATE INDEX IF NOT EXISTS idx_mutation_outbox_status_id ON mutation_outbox (status, id ASC);
```

#### IndexedDB Object Store (Web CSR, Tauri)
- **Store Name:** `mutation_outbox`
- **Key Path:** `id` (`autoIncrement: true`)
- **Indexes:**
  - `by_status`: index on `status`
  - `by_idempotency_key`: unique index on `idempotency_key`
- **Record Schema:**
  ```ts
  interface MutationOutboxEntry {
    id?: number;                         // Auto-incrementing sequence (FIFO order)
    idempotency_key: string;             // Client-generated UUID
    endpoint: string;                    // Target API route (e.g. "/api/v1/events")
    method: "POST" | "PUT" | "DELETE" | "PATCH";
    payload: unknown;                    // Raw request body
    created_at: number;                  // Epoch milliseconds
    retry_count: number;                 // Attempt counter (default 0)
    status: "PENDING" | "IN_FLIGHT" | "FAILED_RETRYABLE" | "FAILED_FATAL";
  }
  ```

---

## 4. Mutation Status Machine

```
   [Enqueue Action]
          │
          ▼
     ┌─────────┐
     │ PENDING │
     └────┬────┘
          │ Dispatch next in queue (FIFO)
          ▼
    ┌───────────┐
    │ IN_FLIGHT │
    └─────┬─────┘
          ├─────────────────────────┬─────────────────────────┐
          │ HTTP 2xx Success        │ Network Drop / 5xx      │ HTTP 4xx Client Error
          ▼                         ▼                         ▼
      [DELETE]             ┌──────────────────┐      ┌──────────────┐
  (Purge from outbox)      │ FAILED_RETRYABLE │      │ FAILED_FATAL │
                           └────────┬─────────┘      └──────────────┘
                                    │ Exponential Backoff
                                    └──────► (Reset to PENDING)
```

1. **`PENDING`**: Request is queued locally and waiting for dispatch.
2. **`IN_FLIGHT`**: Currently being dispatched across the network. Prevents concurrent duplicate sends.
3. **`FAILED_RETRYABLE`**: Dispatch failed due to network outage, DNS failure, or 5xx server error. Will retry using exponential backoff with jitter.
4. **`FAILED_FATAL`**: Dispatch failed with 4xx client/validation error. Halts sequential head blocking and flags action for user remediation.
