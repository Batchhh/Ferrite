# FFI Reference

Ferrite uses [UniFFI](https://mozilla.github.io/uniffi-rs/) in proc-macro mode to bridge Rust and Swift. This document describes every exported type and the conventions used at the boundary.

---

## Overview

The FFI layer lives in `src/rust/ferrite-ffi/`. It depends on `ferrite-pe` and compiles to a static library (`libferrite_ffi.a`). At build time, `uniffi-bindgen` reads the compiled `.a` and generates matching Swift wrappers.

No `.udl` files are used — annotations are applied directly on Rust types.

---

## Exported types

### `DecompilerSession` — Object

The top-level session. Holds a list of loaded assemblies and a counter for unique IDs. Thread-safe (`Arc<Mutex<…>>`).

```swift
// Swift API (generated)
class DecompilerSession {
    init()
    func loadAssembly(path: String) throws -> String
    func removeAssembly(id: String) throws
    func getAssemblies() -> [LoadedAssemblyEntry]
    func getAssemblyInfo(id: String) throws -> AssemblyInfo
    func decompileType(assemblyId: String, typeToken: UInt32) throws -> String
}
```

### `AssemblyInfo` — Record

Top-level metadata for a loaded assembly.

| Swift field | Rust field | Type |
|---|---|---|
| `name` | `name` | `String` |
| `version` | `version` | `String` |
| `targetFramework` | `target_framework` | `String` |
| `namespaces` | `namespaces` | `[NamespaceInfo]` |
| `assemblyReferences` | `assembly_references` | `[String]` |

### `NamespaceInfo` — Record

| Swift field | Rust field | Type |
|---|---|---|
| `name` | `name` | `String` |
| `types` | `types` | `[TypeInfo]` |

### `TypeInfo` — Record

| Swift field | Rust field | Type |
|---|---|---|
| `name` | `name` | `String` |
| `fullName` | `full_name` | `String` |
| `kind` | `kind` | `TypeKind` |
| `token` | `token` | `UInt32` |
| `namespace` | `namespace` | `String` |
| `attributes` | `attributes` | `TypeAttributes` |
| `members` | `members` | `[MemberInfo]` |
| `properties` | `properties` | `[PropertyInfo]` |
| `nestedTypes` | `nested_types` | `[TypeInfo]` |
| `baseType` | `base_type` | `String?` |
| `interfaces` | `interfaces` | `[String]` |
| `attributesList` | `attributes_list` | `[AttributeInfo]` |

### `MemberInfo` — Record

| Swift field | Rust field | Type |
|---|---|---|
| `name` | `name` | `String` |
| `kind` | `kind` | `MemberKind` |
| `token` | `token` | `UInt32` |
| `signature` | `signature` | `String` |
| `methodAttributes` | `method_attributes` | `MethodAttributes?` |
| `fieldAttributes` | `field_attributes` | `FieldAttributes?` |
| `returnType` | `return_type` | `String` |
| `parameters` | `parameters` | `[ParameterInfo]` |
| `attributesList` | `attributes_list` | `[AttributeInfo]` |
| `fieldType` | `field_type` | `String` |
| `constantValue` | `constant_value` | `String?` |

### `PropertyInfo` — Record

| Swift field | Rust field | Type |
|---|---|---|
| `name` | `name` | `String` |
| `token` | `token` | `UInt32` |
| `propertyType` | `property_type` | `String` |
| `getterToken` | `getter_token` | `UInt32?` |
| `setterToken` | `setter_token` | `UInt32?` |
| `attributesList` | `attributes_list` | `[AttributeInfo]` |

### `TypeAttributes` — Record

| Field | Type |
|---|---|
| `visibility` | `Visibility` |
| `isAbstract` | `Bool` |
| `isSealed` | `Bool` |
| `isStatic` | `Bool` |

### `MethodAttributes` — Record

| Field | Type |
|---|---|
| `visibility` | `Visibility` |
| `isStatic` | `Bool` |
| `isVirtual` | `Bool` |
| `isAbstract` | `Bool` |
| `isFinal` | `Bool` |
| `isConstructor` | `Bool` |

### `FieldAttributes` — Record

| Field | Type |
|---|---|
| `visibility` | `Visibility` |
| `isStatic` | `Bool` |
| `isInitOnly` | `Bool` |
| `isLiteral` | `Bool` |

### `ParameterInfo` — Record

| Field | Type |
|---|---|
| `name` | `String` |
| `typeName` | `String` |

### `AttributeInfo` — Record

| Field | Type |
|---|---|
| `name` | `String` |
| `arguments` | `[String]` |

### `LoadedAssemblyEntry` — Record

| Field | Type |
|---|---|
| `id` | `String` |
| `filePath` | `String` |
| `info` | `AssemblyInfo` |

---

## Enums

### `TypeKind`

```swift
enum TypeKind { case `class`, interface, `struct`, `enum`, delegate }
```

Swift keyword variants are backtick-escaped by the Swift compiler.

### `MemberKind`

```swift
enum MemberKind { case method, field, property, event }
```

### `Visibility`

```swift
enum Visibility {
    case `public`, `private`, `internal`, protected,
         protectedInternal, privateProtected
}
```

---

## Error handling

```swift
enum FerriteError: Error {
    case ioError(message: String)
    case parseError(message: String)
    case notFound(message: String)
}
```

All `DecompilerSession` methods that can fail are `throws` in Swift and return a `Result` in Rust. Errors propagate through the UniFFI boundary transparently.

---

## Build artifact locations

| Artifact | Location |
|---|---|
| `libferrite_ffi.a` | `libs/libferrite_ffi.a` |
| `ferrite_ffi.swift` | `src/swift/Ferrite/Generated/ferrite_ffi.swift` |
| `ferrite_ffiFFI.h` | `src/swift/FerriteFFI/include/ferrite_ffiFFI.h` |
| `ferrite_ffiFFI.modulemap` | `src/swift/FerriteFFI/include/module.modulemap` |

All generated files are gitignored and must be produced by `make generate-bindings`.
