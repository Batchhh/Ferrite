#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$PROJECT_DIR/src/rust"
GENERATED_DIR="$PROJECT_DIR/src/swift/Ferrite/Generated"

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    RUST_TARGET="aarch64-apple-darwin"
else
    RUST_TARGET="x86_64-apple-darwin"
fi

SKIP_CHECKS=false
for arg in "$@"; do
    case "$arg" in
        --skip-checks) SKIP_CHECKS=true ;;
    esac
done

cd "$RUST_DIR"

if [ "$SKIP_CHECKS" = false ]; then
    echo "==> Checking formatting..."
    cargo fmt --check

    echo "==> Running Clippy..."
    cargo clippy -- -D warnings

    echo "==> Running tests..."
    cargo test
fi

echo "==> Building Rust crates (release, $RUST_TARGET)..."
RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix $HOME/.cargo/registry/src/=cargo: --remap-path-prefix $HOME/.rustup/toolchains/=rustup: --remap-path-prefix $HOME/=~/" cargo build --release --target "$RUST_TARGET"

echo "==> Generating Swift bindings..."
mkdir -p "$GENERATED_DIR"

cargo run --release --bin uniffi-bindgen generate \
    --library "target/$RUST_TARGET/release/libferrite_ffi.a" \
    --language swift \
    --out-dir "$GENERATED_DIR"

# Copy the static library to libs/
mkdir -p "$PROJECT_DIR/libs"
cp "target/$RUST_TARGET/release/libferrite_ffi.a" "$PROJECT_DIR/libs/libferrite_ffi.a"

# Copy the FFI header into the C target so SwiftPM can find it
FFI_INCLUDE="$PROJECT_DIR/src/swift/FerriteFFI/include"
mkdir -p "$FFI_INCLUDE"
cp "$GENERATED_DIR/ferrite_ffiFFI.h" "$FFI_INCLUDE/ferrite_ffiFFI.h"

# Patch generated Swift for strict concurrency (Swift 6)
sed -i '' 's/^private var initializationResult/nonisolated(unsafe) private var initializationResult/' "$GENERATED_DIR/ferrite_ffi.swift"

echo "==> Done. Generated files in $GENERATED_DIR"
ls -la "$GENERATED_DIR"
