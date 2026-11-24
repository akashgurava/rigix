# Target Agent Guidelines: Apple Presentation Shell (`ui/apple`)

## Framework & Toolchain
- **Stack**: Swift 6, SwiftUI, Swift Concurrency (async/await, Actors).
- **Bundle ID**: Configured via `app_id` (e.g. `com.example.my-app`).
- **Status**: **Optional Target** (`default: false`, opt-in).

## Persistence & State
- **Storage**: GRDB.swift (raw SQLite wrapper with zero ORM overhead).
- **Offline Mirror**: `read_cache` table with endpoint text PK and raw payload string.
- **FIFO Outbox**: `mutation_outbox` table drained by a Swift Actor concurrency worker.
- **Credentials**: Secure Enclave / iOS & macOS Keychain.
