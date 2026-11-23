# 0003: Client Local Storage Engines, Schemas & Offline Outbox

## Context
Rigix client targets (`android`, `apple`, `flutter`, `web`, and `tauri`) require a decoupled, lightweight local persistence architecture for two specific duties:
1. **Offline Read Fallback (`read_cache`)**: Mirroring server responses so users are not presented with empty screens when disconnected.
2. **Offline FIFO Mutation Queue (`mutation_outbox`)**: Reliably buffering user actions while disconnected and dispatching them sequentially upon network restoration without client-side optimistic UI drift.

Evaluating client-side caching interceptors and Stale-While-Revalidate (SWR) revalidation layers against project constraints revealed that:
1. Complex client-side caching interceptors violate the **Single Source of Truth (SSOT)** principle by intercepting server truth and creating duplicate, out-of-sync state machines on client devices.
2. High-level ORMs (e.g. Apple SwiftData, Dart Isar) impose invasive annotations (`@Model`, `@collection`) that severely conflict with official OpenAPI code generators (`swift-openapi-generator`, `openapi-generator-cli`).
3. Shredding OpenAPI responses into relational domain tables creates massive friction: any backend schema change in `utoipa` would force client-side SQLite migrations and manual DTO-to-entity mappers.

## Decision
1. **Authoritative SSOT & Transparent UI**:
   - When online, client UI targets fetch directly from the server. The UI remains completely transparent and avoids complex client-side caching interceptors.
   - Successful server responses are copied to local storage strictly as an **offline fallback**.
   - Advanced UI interceptor/caching optimizations (such as SWR revalidation) are deferred to future offline optimization milestones.
2. **Concrete Storage Engines**:
   - **Android**: Android Jetpack Room with SQLite (`androidx.room`).
   - **Apple (iOS/macOS)**: GRDB.swift with SQLite.
   - **Flutter**: Drift with SQLite (`drift` / `drift_flutter`).
   - **Web CSR & Tauri**: Unified `idb` (IndexedDB), ensuring a 100% identical web bundle (`ui/web/dist`) across browser and desktop WebView without IPC storage divergence.
3. **Raw OpenAPI JSON Envelope Model**:
   - Eliminate object-relational mapping (ORM) table shredding. Storage stores raw unparsed JSON payloads keyed by endpoint URL, delegating 100% of typed serialization and deserialization to official OpenAPI code generators:
     - Web & Tauri: `openapi-typescript` + `openapi-fetch`
     - Apple: `swift-openapi-generator` (native `Codable` structs)
     - Android: `openapi-generator-cli` with Kotlin + `kotlinx.serialization`
     - Flutter: `openapi-generator-cli` with Dart + `json_serializable`
4. **Standardized Schemas**:
   - **`read_cache` (Offline Fallback Mirror)**:
     - `endpoint_url TEXT PRIMARY KEY` (normalized URL and query string)
     - `payload TEXT NOT NULL` (raw unparsed JSON response)
     - `updated_at INTEGER NOT NULL` (epoch milliseconds)
   - **`mutation_outbox` (Sequential FIFO Queue)**:
     - `id INTEGER PRIMARY KEY AUTOINCREMENT` (monotonically increasing FIFO sequence)
     - `idempotency_key TEXT NOT NULL UNIQUE` (client-generated UUID sent in `Idempotency-Key` header)
     - `endpoint TEXT NOT NULL` (target URL path)
     - `method TEXT NOT NULL` (`POST`, `PUT`, `DELETE`, `PATCH`)
     - `payload TEXT NOT NULL` (raw JSON request body)
     - `created_at INTEGER NOT NULL` (epoch milliseconds)
     - `retry_count INTEGER NOT NULL DEFAULT 0`
     - `status TEXT NOT NULL DEFAULT 'PENDING'` (`PENDING`, `IN_FLIGHT`, `FAILED_RETRYABLE`, `FAILED_FATAL`)
5. **Storage Sandboxing & Security**:
   - Sensitive JWT tokens (refresh tokens, session keys) are kept strictly in platform-native secure storage (Keychain, Keystore, SecureStorage).
   - `read_cache` and `mutation_outbox` live in standard OS-sandboxed SQLite / IndexedDB without full-database SQLCipher overhead.

## Consequences
- Zero ORM impedance mismatch: backend `utoipa` schema updates require zero client-side SQLite/IndexedDB migrations.
- Total cross-platform symmetry across Android, Apple, Flutter, Web, and Tauri.
- The UI layer remains completely transparent, predictable, and aligned with the server-authoritative SSOT principle.
- Reliable offline capabilities: screens load cached data when disconnected, and offline user mutations queue safely without data loss.
