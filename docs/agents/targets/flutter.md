# Target Agent Guidelines: Flutter Presentation Shell (`ui/flutter`)

## Framework & Toolchain
- **Stack**: Dart, Flutter (Multi-platform mobile & desktop).
- **Status**: **Optional Target** (`default: false`, opt-in).

## Persistence & State
- **Storage**: Drift (type-safe reactive persistence for SQLite).
- **Offline Mirror**: `read_cache` table with raw string envelopes.
- **FIFO Outbox**: `mutation_outbox` table with sequential FIFO processing.
- **Credentials**: `flutter_secure_storage` backed by platform-native keychains.
