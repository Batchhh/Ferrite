# Architecture

Ferrite is split into two halves: a Rust backend that understands .NET binary formats, and a Swift/SwiftUI frontend. The two halves communicate through a generated FFI boundary powered by [UniFFI](https://mozilla.github.io/uniffi-rs/).

---

## Layer overview

```mermaid
graph TD
    A["SwiftUI App (macOS 26)"] --> B["DecompilerService (@Observable @MainActor)"]
    B --> C["UniFFI-generated Swift bindings (Generated/)"]
    C --> D["ferrite-ffi (Rust staticlib)"]
    D --> E["ferrite-pe (Rust library)"]

    style A fill:#1e1e2e,stroke:#89b4fa,color:#cdd6f4
    style B fill:#1e1e2e,stroke:#89b4fa,color:#cdd6f4
    style C fill:#1e1e2e,stroke:#a6e3a1,color:#cdd6f4
    style D fill:#1e1e2e,stroke:#fab387,color:#cdd6f4
    style E fill:#1e1e2e,stroke:#fab387,color:#cdd6f4
```

| Layer | Language | Role |
|---|---|---|
| `ferrite-pe` | Rust | Parses PE headers, CLR metadata tables, IL bytecode; lifts IL → C# AST |
| `ferrite-ffi` | Rust | UniFFI-annotated boundary; compiles to `libferrite_ffi.a` |
| `FerriteFFI` | C | Thin module target exposing the generated header to Swift |
| `Ferrite` | Swift/SwiftUI | UI: sidebar, code view, search, project management |

---

## Assembly parsing pipeline

```mermaid
flowchart LR
    bytes[Raw bytes] --> goblin["goblin (PE headers)"]
    goblin --> clr["CLR header (IMAGE_COR20_HEADER)"]
    clr --> root["Metadata root (BSJB magic)"]
    root --> streams[Stream headers]
    streams --> tables["#~ tables (TypeDef · MethodDef · Field · Param · NestedClass · CustomAttribute)"]
    streams --> strings["#Strings heap (UTF-8 identifiers)"]
    streams --> blob["#Blob heap (Signatures)"]
    tables --> assembly["Assembly model (TypeDef · MethodDef · FieldDef)"]
    strings --> assembly
    blob --> assembly
```

Key facts:
- Table rows are 1-indexed. Tokens encode the table ID in the high byte: `TypeDef=0x02`, `MethodDef=0x06`, `Field=0x04`.
- `TypeDef` row 1 is always the `<Module>` pseudo-type and is filtered out.
- The `NestedClass` table maps child → parent; after parsing, nested types are removed from the top-level list and attached to their parent's `nested_types` vec.

### TypeKind detection

| Condition | Kind |
|---|---|
| `flags & 0x20 != 0` | `Interface` |
| extends `System.Enum` | `Enum` |
| extends `System.ValueType` | `Struct` |
| extends `System.MulticastDelegate` | `Delegate` |
| otherwise | `Class` |

---

## Decompiler pipeline

```mermaid
flowchart LR
    token[MethodDef token] --> body["method_body (parse raw IL)"]
    body --> insns["instructions (decode opcodes)"]
    insns --> stack["stack/simulation (IL → AstNode list)"]
    stack --> cfg["control_flow (CFG → if / while / try-catch)"]
    cfg --> patterns["patterns (foreach · LINQ · null-coalescing · lambdas · using)"]
    patterns --> emit["emit (AstNode → C# text)"]
```

Patterns run in registration order; more specific patterns are registered before more general ones.

---

## FFI boundary

```mermaid
graph LR
    swift["Swift (DecompilerService)"] -->|calls| gen["Generated ferrite_ffi.swift"]
    gen -->|C ABI| header[ferrite_ffiFFI.h]
    header -->|links| lib[libferrite_ffi.a]
    lib -->|Rust| session["DecompilerSession (Arc Mutex)"]

    style swift fill:#1e1e2e,stroke:#89b4fa,color:#cdd6f4
    style gen fill:#1e1e2e,stroke:#a6e3a1,color:#cdd6f4
    style header fill:#1e1e2e,stroke:#a6e3a1,color:#cdd6f4
    style lib fill:#1e1e2e,stroke:#fab387,color:#cdd6f4
    style session fill:#1e1e2e,stroke:#fab387,color:#cdd6f4
```

UniFFI proc-macro mode — no `.udl` files. Annotations are applied directly on Rust types:

| Annotation | Used for |
|---|---|
| `#[derive(uniffi::Object)]` | `DecompilerSession` — Arc-wrapped, reference-counted across FFI |
| `#[derive(uniffi::Record)]` | Struct types passed by value (cloned at boundary) |
| `#[derive(uniffi::Enum)]` | Enums |
| `#[derive(uniffi::Error)]` | `FerriteError` — propagated as Swift `throws` |

Naming: Rust `snake_case` fields → Swift `camelCase` automatically (e.g. `full_name` → `fullName`). Swift keyword enum variants are backtick-escaped: `` .`class` ``, `` .`struct` ``.

---

## Swift app

### State management

```mermaid
graph TD
    app[FerriteApp] --> env{Environment}
    env --> ds["DecompilerService (FFI session · selection · assemblies)"]
    env --> ps["ProjectService (project file · persistence)"]
    env --> ss["SearchService (in-memory index · fuzzy scoring)"]
    ds --> views["Views (read-only consumers)"]
    ps --> views
    ss --> views
```

All service properties are `@MainActor`. FFI calls run in `Task.detached` and marshal results back with `await MainActor.run`.

### View hierarchy

```mermaid
graph TD
    cv[ContentView] --> atv["AssemblyTreeView (sidebar)"]
    cv --> cpv["CodePreviewView (decompiled C#)"]
    cv --> dv["DetailView (assembly / namespace summary)"]
    cv --> sp["SearchPanel (Cmd+K overlay)"]
    cv --> pmo[ProjectManagerOverlay]
    cv --> npo[NewProjectOverlay]
    cv --> wv["WelcomeView (no assembly loaded)"]
    atv --> anv["AssemblyNodeView (tree rows)"]
    cpv --> bb[BreadcrumbBar]
    cpv --> ctv["CodeTextView (NSTextView + syntax highlight)"]
```
