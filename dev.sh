#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

WASM_TARGET="wasm32-unknown-unknown"

ensure_wasm_target() {
    if ! rustup target list | grep -q "${WASM_TARGET} (installed)"; then
        echo "Installing missing target: ${WASM_TARGET}..."
        rustup target add "${WASM_TARGET}"
    fi
}

case "${1:-build}" in
    fmt)
        shift || true
        case "${1:-}" in
            --check)
                cargo fmt --all -- --check
                ;;
            "")
                cargo fmt --all
                ;;
            *)
                echo "Usage: ./dev.sh fmt [--check]"
                exit 1
                ;;
        esac
        ;;
    lint)
        shift || true
        ensure_wasm_target
        case "${1:-}" in
            --check)
                echo "==> Linting native (all features)..."
                cargo clippy --workspace --all-targets --all-features -- -D warnings
                echo "==> Linting wasm (default macros)..."
                cargo clippy --package rigix --target "${WASM_TARGET}" -- -D warnings
                ;;
            "")
                echo "==> Fixing and linting native (all features)..."
                cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
                cargo clippy --workspace --all-targets --all-features -- -D warnings
                echo "==> Fixing and linting wasm (default macros)..."
                cargo clippy --package rigix --target "${WASM_TARGET}" --fix --allow-dirty --allow-staged
                cargo clippy --package rigix --target "${WASM_TARGET}" -- -D warnings
                ;;
            *)
                echo "Usage: ./dev.sh lint [--check]"
                exit 1
                ;;
        esac
        ;;
    build)
        ensure_wasm_target
        echo "==> Building native (all features)..."
        cargo build --workspace --all-features
        echo "==> Building wasm (default macros)..."
        cargo build --package rigix --target "${WASM_TARGET}"
        ;;
    test)
        ensure_wasm_target
        echo "==> Testing native (all features)..."
        cargo test --workspace --all-features
        echo "==> Checking wasm targets (default macros)..."
        cargo check --package rigix --target "${WASM_TARGET}"
        ;;
    clean)
        cargo clean
        ;;
    check)
        ensure_wasm_target
        echo "==> Checking formatting..."
        cargo fmt --all -- --check
        echo "==> Linting native (all features)..."
        cargo clippy --workspace --all-targets --all-features -- -D warnings
        echo "==> Linting wasm (default macros)..."
        cargo clippy --package rigix --target "${WASM_TARGET}" -- -D warnings
        echo "==> Testing native (all features)..."
        cargo test --workspace --all-features
        echo "==> Checking wasm (default macros)..."
        cargo check --package rigix --target "${WASM_TARGET}"
        ;;
    *)
        echo "Usage: ./dev.sh [build|test|lint [--check]|fmt [--check]|clean|check]"
        exit 1
        ;;
esac
