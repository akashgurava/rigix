# Target Agent Guidelines: Universal Multi-Platform Shell (`ui/tauri`)

## Framework & Toolchain
- **Stack**: Tauri 2.0 (Rust universal shell wrapping `ui/web/dist`).
- **Platforms**: Multi-target shell supporting macOS, iOS, Android, Linux, and Windows.
- **Identifier**: Configured via `app_id` in `tauri.conf.json`.
- **Status**: **Universal Fallback Target** (`default: false`, opt-in). Serves as the complete multi-platform solution (mobile and desktop) for projects that do not require dedicated native Kotlin/Swift presentation shells.

## Packaging & Asset Delivery
- **Frontend Dist**: Points to local web build `../../web/dist`.
- **Offline Boot**: Native webview loads local embedded static HTML/JS/CSS across all platforms without requiring an active network connection.
- **Backend Connection**: Runtime-configurable API base URL with local IndexedDB storage.

## Target Build & Run Verification
Agents working on `ui/tauri` must verify both compilation and live runtime execution:
1. **Desktop (macOS)**: `cargo tauri dev` or `cargo tauri build` running locally.
2. **Mobile (iOS)**: `cargo tauri ios dev` running on an active iOS Simulator.
3. **Mobile (Android)**: `cargo tauri android dev` running on an active Android Emulator.
4. **Linux**: Containerized build verification via Docker.
5. **Windows**: Cross-compilation typecheck (`cargo check --target x86_64-pc-windows-msvc`) with final MSI packaging verified in CI.
