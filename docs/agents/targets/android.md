# Target Agent Guidelines: Android Presentation Shell (`ui/android`)

## Framework & Toolchain
- **Stack**: Kotlin, Jetpack Compose, Coroutines / StateFlow.
- **Package / Namespace**: Configured via sanitized `package_name` (e.g. `com.example.app_name`).
- **Status**: **Default Target** (`default: true` in template profile).

## Persistence & State
- **Storage**: Android Jetpack Room with Kotlin Symbol Processing (KSP).
- **Offline Mirror**: `read_cache` entity storing endpoint URLs and raw JSON response text.
- **FIFO Outbox**: `mutation_outbox` Room entity with ascending integer primary key.
- **Concurrency**: Singleton background worker backed by Coroutine Mutex.
- **Credentials**: Sensitive JWT tokens stored in Android Keystore / EncryptedSharedPreferences.
