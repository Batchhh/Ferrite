# Building Ferrite

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Xcode | 16+ | Mac App Store |
| Xcode Command Line Tools | 16+ | `xcode-select --install` |
| Rust | 1.80+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| xcodegen | latest | `brew install xcodegen` |

The build script installs `uniffi-bindgen` automatically as a Cargo binary tool (via the `uniffi-bindgen` binary target inside `ferrite-ffi`).

---

## Quick start

```bash
git clone https://github.com/Batchhh/Ferrite.git
cd Ferrite
make all
open Ferrite.xcodeproj
```

Then press **Cmd+R** in Xcode to run.

---

## Build steps explained

### `make all`

Runs `generate-bindings` then `xcode`:

1. **`scripts/build-rust.sh`** — the main build script:
   - Detects the host architecture (`arm64` or `x86_64`).
   - Runs `cargo fmt --check` and `cargo clippy -- -D warnings` (skipped with `--skip-checks`).
   - Runs `cargo test`.
   - Compiles a release build for the detected target (`aarch64-apple-darwin` or `x86_64-apple-darwin`).
   - Runs `uniffi-bindgen generate` to produce `ferrite_ffi.swift` and `ferrite_ffiFFI.h`.
   - Copies `libferrite_ffi.a` → `libs/`.
   - Copies `ferrite_ffiFFI.h` → `src/swift/FerriteFFI/include/`.
   - Patches the generated Swift for Swift 6 strict concurrency.

2. **`xcodegen generate`** — regenerates `Ferrite.xcodeproj` from `project.yml`.

### `make build-rust`

Same as `generate-bindings` but passes `--skip-checks` (no fmt/clippy/test). Useful during rapid iteration.

### `make clean`

```bash
cd src/rust && cargo clean
rm -rf src/swift/Ferrite/Generated
rm -rf .build DerivedData
```

---

## Rust-only workflow

```bash
cd src/rust
cargo build --release --target aarch64-apple-darwin   # or x86_64-apple-darwin
cargo test
cargo clippy -- -D warnings
cargo fmt
```

---

## Xcode build settings reference

Relevant settings in `project.yml` (propagated to `Ferrite.xcodeproj`):

| Setting | Value |
|---|---|
| `PRODUCT_BUNDLE_IDENTIFIER` | `com.ferrite.app` |
| `MACOSX_DEPLOYMENT_TARGET` | `26.0` |
| `LIBRARY_SEARCH_PATHS` | `$(SRCROOT)/libs` |
| `OTHER_LDFLAGS` | `-lferrite_ffi` |
| `SWIFT_INCLUDE_PATHS` | `$(SRCROOT)/src/swift/FerriteFFI/include` |

The Xcode project links `libferrite_ffi.a` from `libs/` at link time. The C module target `FerriteFFI` exposes `ferrite_ffiFFI.h` so Swift can `import ferrite_ffiFFI`.

---

## Regenerating the Xcode project

Run `xcodegen generate` (or `make xcode`) any time you change `project.yml` — e.g. after adding a new Swift source file or changing build settings. You do **not** need to do this for Rust changes.

---

## Troubleshooting

### `libferrite_ffi.a` not found

Run `make generate-bindings` to rebuild the Rust static library and copy it to `libs/`.

### `ferrite_ffi.swift` missing

Same fix: run `make generate-bindings`. The generated Swift files are gitignored and must be produced locally.

### Architecture mismatch

If you see linker errors about `arm64` vs `x86_64`, ensure the Rust target matches your Mac's architecture:

```bash
uname -m   # arm64 or x86_64
```

The build script detects this automatically, but if you compiled manually make sure you used the right `--target` flag.

### Swift 6 concurrency errors in generated code

The build script patches the generated `ferrite_ffi.swift` for Swift 6 strict concurrency. If you manually ran `uniffi-bindgen` without the script, apply the patch:

```bash
sed -i '' 's/^private var initializationResult/nonisolated(unsafe) private var initializationResult/' \
    src/swift/Ferrite/Generated/ferrite_ffi.swift
```
