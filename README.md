<p align="center">
  <img src="docs/images/icon.png" width="100" alt="Ferrite icon">
  <br><br>
  <b><samp>FERRITE</samp></b>
  <br>
  <sub>Native macOS .NET decompiler — built with <b>Rust</b> and <b>Swift</b>.</sub>
  <br><br>
  <img src="https://img.shields.io/badge/version-0.2.0-222?style=for-the-badge&logoColor=white" alt="Version">
  <img src="https://img.shields.io/badge/macOS-26%2B-222?style=for-the-badge&logoColor=white" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-222?style=for-the-badge&logoColor=white" alt="License">
  <a href="https://x.com/BatchhRE"><img src="https://img.shields.io/badge/by%20batchh-222?style=for-the-badge&logo=x&logoColor=white" alt="X"></a>
</p>

<br>

<p align="center">
  <img src="docs/images/inspector.png" width="720" alt="Ferrite inspector view">
</p>

<br>

Features
--------

- **C# decompilation** — async/await, generics, lambdas, LINQ, pattern matching and more
- **IL disassembly** — full ECMA-335 opcode view with syntax highlighting and clickable type references
- **Assembly browser** — sidebar tree: assembly → namespace → type → member
- **Fuzzy search** — `Cmd+K` across all loaded types and members
- **Multi-assembly projects** — group assemblies into projects that persist between sessions
- **Drag-and-drop** — drop `.dll` / `.exe` directly onto the window
- **Code export** — `Cmd+E` saves the current view as a `.cs` file
- **Memory-mapped I/O** — fast loads, minimal RAM usage
- **Lazy loading** — summaries on startup, full details fetched on demand

How to install
--------------

Download the latest `.dmg` from [Releases](../../releases), mount it, and drag **Ferrite.app** to `/Applications`.

> **Gatekeeper:** Ferrite is not notarized. Run `xattr -cr /Applications/Ferrite.app` or right-click → **Open** to bypass the warning.

How to build
------------

Requirements: macOS 26+, Xcode 16+, Rust 1.80+, `xcodegen` (`brew install xcodegen`)

```bash
git clone https://github.com/Batchhh/Ferrite.git
cd Ferrite
make all
open Ferrite.xcodeproj   # then Cmd+R
```

See [docs/building.md](docs/building.md) for details.

Architecture
------------

```
SwiftUI app  ──UniFFI──▸  Rust static library
(src/swift/)              (src/rust/)
```

See [docs/architecture.md](docs/architecture.md) for a full breakdown.

How to contribute
-----------------

See [CONTRIBUTING.md](CONTRIBUTING.md).

License
-------

MIT — see [LICENSE](LICENSE).
