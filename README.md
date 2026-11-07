# Rigix

Rigix is an opinionated, compile-time-enforced Rust application framework and metaprogramming DSL designed to eliminate multi-crate boilerplate and manual policing of standards.

## Crates & Features

- **`crates/rigix-macros`**: Procedural macros (`#[entity]`, `#[route]`).
- **`crates/rigix`**: Unified facade crate.
  - **`macros` (Default)**: Procedural macros and core serialization types. Targets both native and WebAssembly (`wasm32-unknown-unknown`).
  - **`server` (Add-on)**: Axum runtime host (`AppHost`), CLI options, Swagger UI, and tracing. Native only.
  - **`auth` (Add-on)**: Argon2 password hashing and session management. Native only.
  - **`db` (Add-on)**: SQLite persistence and SQLx integration. Native only.
  - **`full` (Add-on)**: Enables `macros`, `server`, `auth`, and `db`.

## Development

The workspace runner automatically builds, lints, and checks both **native** and **wasm32** targets:

```bash
./dev.sh build         # Build native (all features) and wasm (default macros)
./dev.sh test          # Run native tests and verify wasm check
./dev.sh lint          # Clippy with auto-fix on native and wasm
./dev.sh lint --check  # Verify clippy without applying fixes on native and wasm
./dev.sh fmt           # Format code
./dev.sh fmt --check   # Check formatting
./dev.sh check         # Run fmt --check, lint --check, test, and wasm check
./dev.sh clean         # Clean build artifacts
```

See [HANDOFF.md](HANDOFF.md) for architectural specifications.
