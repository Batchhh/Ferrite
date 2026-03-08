# Contributing to Ferrite

Thank you for your interest in contributing! This document covers the development workflow, code conventions, and how to add new features.

## Table of Contents

- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Rust Conventions](#rust-conventions)
- [Swift Conventions](#swift-conventions)
- [Adding a Decompiler Pattern](#adding-a-decompiler-pattern)
- [Adding a New FFI Type](#adding-a-new-ffi-type)
- [Testing](#testing)
- [Submitting a Pull Request](#submitting-a-pull-request)

---

## Getting Started

1. Fork and clone the repository.
2. Install prerequisites: Rust, Xcode 16+, `xcodegen`, and a C compiler (ships with Xcode CLT).
3. Run `make all` to build everything and open `Ferrite.xcodeproj` in Xcode.

```bash
brew install xcodegen
make all
open Ferrite.xcodeproj
```

---

## Project Structure

```
Ferrite/
├── src/
│   ├── rust/
│   │   ├── ferrite-pe/     # PE parser and C# decompiler
│   │   └── ferrite-ffi/    # UniFFI boundary (staticlib)
│   └── swift/
│       ├── Ferrite/        # SwiftUI app
│       └── FerriteFFI/     # C module target (generated header)
├── scripts/
│   └── build-rust.sh       # Compiles Rust + generates Swift bindings
├── libs/                   # Built libferrite_ffi.a (gitignored output)
├── project.yml             # XcodeGen spec
├── Makefile
└── CLAUDE.md               # AI assistant context
```

---

## Development Workflow

### After changing Rust code

```bash
make all          # recompile + regenerate bindings + regen Xcode project
# Then Cmd+B in Xcode
```

### Rust-only iteration (skip Swift regeneration)

```bash
cd src/rust
cargo test        # run unit tests
cargo clippy -- -D warnings
cargo fmt
```

### Swift-only iteration

Edit Swift files and press Cmd+B in Xcode — no Rust rebuild needed unless the FFI changed.

---

## Rust Conventions

- Run `cargo fmt` before committing. CI enforces formatting with `--check`.
- `cargo clippy -- -D warnings` must pass with zero warnings.
- All public functions in `ferrite-pe` should have doc comments.
- No `unwrap()` outside of tests; use `?` or explicit error mapping.
- Error types live in `assembly/mod.rs` (`PeError`) and `ferrite-ffi/src/types.rs` (`FerriteError`).

---

## Swift Conventions

- All UI state lives in `@Observable` services (`DecompilerService`, `ProjectService`, `SearchService`).
- Views must remain side-effect-free; trigger mutations via service methods.
- Use `@MainActor` for all service methods that touch UI state.
- Prefer `Task.detached` for FFI calls; dispatch results back with `await MainActor.run`.
- Do not import `ferrite_ffi` directly in views — go through `DecompilerService`.

---

## Adding a Decompiler Pattern

Decompiler patterns are in `src/rust/ferrite-pe/src/decompiler/patterns/`. Each file implements a specific C# pattern recogniser (e.g. `loops_foreach.rs`, `null_coalescing.rs`).

1. Create `src/rust/ferrite-pe/src/decompiler/patterns/my_pattern.rs`.
2. Implement a function that takes a slice of `AstNode` and returns a replacement if the pattern matches.
3. Register it in `patterns/mod.rs`.
4. Add a test in `patterns/tests.rs` using a real or synthetic IL byte slice.

Patterns run in order; put more specific patterns before more general ones.

---

## Adding a New FFI Type

1. Add the Rust type (with `#[derive(uniffi::Record)]` or `#[derive(uniffi::Enum)]`) in `src/rust/ferrite-ffi/src/types.rs`.
2. Add conversion logic in `src/rust/ferrite-ffi/src/convert.rs`.
3. Expose it through a method on `DecompilerSession` in `src/rust/ferrite-ffi/src/lib.rs`.
4. Run `make all` — the Swift bindings regenerate automatically.
5. Consume the new type in `DecompilerService.swift` or the relevant view.

Field naming: Rust `snake_case` fields become Swift `camelCase` in generated bindings automatically.

---

## Testing

### Rust tests

```bash
cd src/rust
cargo test                      # all crates
cargo test -p ferrite-pe        # just the parser/decompiler
```

Tests live in:
- `src/rust/ferrite-pe/src/assembly/tests.rs` — metadata parsing
- `src/rust/ferrite-pe/src/decompiler/tests.rs` — end-to-end decompilation
- `src/rust/ferrite-pe/src/decompiler/patterns/tests.rs` — individual patterns

### Swift tests

There are no automated Swift tests yet. Manual testing:
1. Load a real-world `.dll` (e.g. from the .NET SDK or NuGet).
2. Verify the sidebar tree, decompiled output, and search results.

---

## Submitting a Pull Request

1. Branch off `main`: `git checkout -b feat/my-feature`.
2. Make your changes. Ensure `cargo fmt --check` and `cargo clippy -- -D warnings` pass.
3. Add tests for any new Rust logic.
4. Open a PR against `main` with a clear description of what changed and why.
5. A CI run will build the Rust crates and run tests; it must pass before merge.

**Commit style:** short imperative subject line, e.g. `add foreach loop pattern`, `fix null-coalescing precedence`.
