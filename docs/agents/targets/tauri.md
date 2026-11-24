# Target Agent Guidelines: Desktop Shell (`ui/tauri`)

## Framework & Toolchain
- **Stack**: Tauri 2.0 (Rust desktop shell wrapping `ui/web/dist`).
- **Identifier**: Configured via `app_id` in `tauri.conf.json`.
- **Status**: **Optional Target** (`default: false`, opt-in).

## Packaging & Asset Delivery
- **Frontend Dist**: Points to local web build `../../web/dist`.
- **Offline Boot**: Native desktop webview loads local embedded static HTML/JS/CSS without requiring an active network connection.
- **Backend Connection**: Runtime-configurable API base URL with local IndexedDB storage.
