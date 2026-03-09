import SwiftUI

// MARK: - Type Name Parsing Helpers

/// Recursively parse a type name into tokens, handling:
/// - `Type[]` (arrays)
/// - `Type?` (nullables)
/// - `Outer<Inner, Inner2>` (generics)
/// - Nested combinations like `List<int[]>`
func parseTypeTokens(_ typeName: String, into tokens: inout [CodeToken]) {
    var name = typeName

    // Strip trailing array brackets: `Type[][]` → parse `Type` then append `[][]`
    var arraySuffix = ""
    while name.hasSuffix("[]") {
        arraySuffix += "[]"
        name = String(name.dropLast(2))
    }

    // Strip trailing nullable: `Type?` → parse `Type` then append `?`
    var nullableSuffix = ""
    if name.hasSuffix("?") {
        nullableSuffix = "?"
        name = String(name.dropLast(1))
    }

    // Check for generic: find top-level `<` ... `>`
    if let openIdx = findTopLevelGenericOpen(name) {
        let outerName = String(name[name.startIndex..<openIdx])
        let afterOpen = name.index(after: openIdx)
        let innerContent = String(name[afterOpen..<name.index(before: name.endIndex)])

        if isBuiltinType(outerName) {
            tokens.append(.keyword(outerName))
        } else {
            tokens.append(.type_(outerName))
        }

        tokens.append(.punct("<"))

        // Split generic arguments at top-level commas
        let args = splitGenericArgs(innerContent)
        for (i, arg) in args.enumerated() {
            if i > 0 { tokens.append(.plain(", ")) }
            parseTypeTokens(arg.trimmingCharacters(in: .whitespaces), into: &tokens)
        }

        tokens.append(.punct(">"))
    } else {
        if isBuiltinType(name) {
            tokens.append(.keyword(name))
        } else {
            tokens.append(.type_(name))
        }
    }

    if !nullableSuffix.isEmpty { tokens.append(.punct(nullableSuffix)) }
    if !arraySuffix.isEmpty { tokens.append(.punct(arraySuffix)) }
}

/// Find the index of the first top-level `<` in a type name (not nested).
/// Returns nil if no generic bracket found or if the name doesn't end with `>`.
func findTopLevelGenericOpen(_ name: String) -> String.Index? {
    guard name.hasSuffix(">") else { return nil }
    var depth = 0
    for idx in name.indices {
        let ch = name[idx]
        if ch == "<" && depth == 0 { return idx }
        if ch == "<" { depth += 1 }
        if ch == ">" { depth -= 1 }
    }
    return nil
}

/// Split generic arguments at top-level commas (respecting nested `<>`).
func splitGenericArgs(_ content: String) -> [String] {
    var args: [String] = []
    var current = ""
    var depth = 0
    for ch in content {
        if ch == "<" { depth += 1 }
        if ch == ">" { depth -= 1 }
        if ch == "," && depth == 0 {
            args.append(current)
            current = ""
        } else {
            current.append(ch)
        }
    }
    args.append(current)
    return args
}

func isBuiltinType(_ name: String) -> Bool {
    switch name {
    case "void", "bool", "char", "sbyte", "byte", "short", "ushort",
         "int", "uint", "long", "ulong", "float", "double", "decimal",
         "string", "object":
        return true
    default:
        return false
    }
}
