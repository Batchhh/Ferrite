import SwiftUI

// MARK: - Line Builders

/// Build a single tokenized line for a field declaration.
/// Backing fields are rendered as auto-properties instead.
func fieldLine(_ field: MemberInfo, indent: String) -> CodeLine {
    // If it's a backing field, display as auto-property
    if let propName = backingFieldPropertyName(field.name) {
        return propertyLine(propName, field: field, hasGetter: true, hasSetter: true, indent: indent)
    }
    var t: [CodeToken] = []
    if !indent.isEmpty { t.append(.plain(indent)) }
    if let a = field.fieldAttributes {
        appendVisibility(&t, a.visibility)
        if a.isStatic  { t.append(.keyword("static"));   t.append(.space) }
        if a.isLiteral { t.append(.keyword("const"));    t.append(.space) }
        else if a.isInitOnly { t.append(.keyword("readonly")); t.append(.space) }
    }
    let ft = field.fieldType.isEmpty ? "object" : field.fieldType
    parseTypeTokens(ft, into: &t)
    t.append(.space)
    t.append(.field(field.name))
    if let val = field.constantValue {
        t.append(.plain(" = "))
        appendConstantValueToken(&t, val)
    }
    t.append(.punct(";"))
    return .init(tokens: t)
}

/// Append a constant value with the appropriate syntax coloring.
func appendConstantValueToken(_ tokens: inout [CodeToken], _ value: String) {
    if value == "true" || value == "false" || value == "null" {
        tokens.append(.keyword(value))
    } else if value.hasPrefix("\"") {
        tokens.append(.string(value))
    } else if value.hasPrefix("'") {
        tokens.append(.string(value))
    } else {
        // Numeric value
        tokens.append(.number(value))
    }
}

/// Build tokenized lines for a method declaration and its body stub.
///
/// When `expanded` is true the body is shown inline (collapsed to `{ ... }` when false).
/// Handles constructors, conversion operators, and explicit interface implementations.
func methodLines(_ method: MemberInfo, typeName: String, typeKind: TypeKind, indent: String, expanded: Bool = true) -> [CodeLine] {
    var t: [CodeToken] = []
    if !indent.isEmpty { t.append(.plain(indent)) }

    let isCtor      = method.methodAttributes?.isConstructor ?? false
    let isStaticCtor = method.name == ".cctor"
    let isOpImplicit = method.name == "op_Implicit"
    let isOpExplicit = method.name == "op_Explicit"

    let overrideNames: Set<String> = ["Equals", "GetHashCode", "ToString", "Finalize"]
    let isOverride = method.methodAttributes?.isVirtual == true && overrideNames.contains(method.name)

    if let a = method.methodAttributes {
        if isStaticCtor {
            t.append(.keyword("static")); t.append(.space)
        } else if isOpImplicit || isOpExplicit {
            appendVisibility(&t, a.visibility)
            t.append(.keyword("static")); t.append(.space)
        } else {
            appendVisibility(&t, a.visibility)
            if a.isStatic { t.append(.keyword("static")); t.append(.space) }
            if a.isAbstract {
                t.append(.keyword("abstract")); t.append(.space)
            } else if isOverride {
                t.append(.keyword("override")); t.append(.space)
            } else if a.isVirtual && !a.isFinal {
                t.append(.keyword("virtual")); t.append(.space)
            }
            // `sealed` is only meaningful on class members; structs implicitly seal interface implementations.
            if a.isFinal && a.isVirtual && !isOverride && typeKind == .`class` { t.append(.keyword("sealed")); t.append(.space) }
        }
    }

    let returnType = method.returnType.isEmpty ? "void" : method.returnType

    if isCtor {
        t.append(.method(typeName))
    } else if isOpImplicit {
        t.append(.keyword("implicit")); t.append(.space)
        t.append(.keyword("operator")); t.append(.space)
        appendTypeName(&t, returnType)
    } else if isOpExplicit {
        t.append(.keyword("explicit")); t.append(.space)
        t.append(.keyword("operator")); t.append(.space)
        appendTypeName(&t, returnType)
    } else {
        appendTypeName(&t, returnType)
        t.append(.space)
        appendExplicitInterfaceMethod(&t, method.name)
    }

    t.append(.punct("("))
    for (i, param) in method.parameters.enumerated() {
        if i > 0 { t.append(.plain(", ")) }
        appendTypeName(&t, param.typeName)
        t.append(.space)
        t.append(.plain(param.name))
    }
    t.append(.punct(")"))

    if method.methodAttributes?.isAbstract == true || typeKind == .interface {
        return [.init(tokens: t + [.punct(";")])]
    }

    let link = "ferrite://method/\(method.token)"
    if expanded {
        var result: [CodeLine] = [.init(tokens: t)]
        result.append(.init(tokens: [
            .plain(indent),
            CodeToken(text: "{", color: .codeCollapsed, typeName: link)
        ]))
        let bodyLines = methodBodyLines(returnType: returnType, indent: indent)
        if bodyLines.count > 2 {
            result.append(contentsOf: bodyLines[1..<(bodyLines.count - 1)])
        }
        result.append(.init(tokens: [
            .plain(indent),
            CodeToken(text: "}", color: .codeCollapsed, typeName: link)
        ]))
        return result
    } else {
        // Collapsed: declaration + { ... }
        t.append(.space)
        t.append(CodeToken(text: "{ ... }", color: .codeCollapsed, typeName: link))
        return [.init(tokens: t)]
    }
}

/// Build the opening `{`, stub body, and closing `}` lines for a method.
func methodBodyLines(returnType: String, indent: String) -> [CodeLine] {
    if returnType == "void" {
        return [
            .init(tokens: [.plain(indent), .punct("{")]),
            .init(tokens: [.plain(indent), .punct("}")]),
        ]
    } else {
        var bodyTokens: [CodeToken] = [.plain(indent), .plain("    "), .controlFlow("return"), .space]
        bodyTokens.append(contentsOf: defaultReturnTokens(returnType))
        bodyTokens.append(.punct(";"))
        return [
            .init(tokens: [.plain(indent), .punct("{")]),
            .init(tokens: bodyTokens),
            .init(tokens: [.plain(indent), .punct("}")]),
        ]
    }
}

/// Return the appropriate default-value tokens for a `return` stub (e.g. `0`, `false`, `null`).
func defaultReturnTokens(_ returnType: String) -> [CodeToken] {
    switch returnType {
    case "int", "byte", "sbyte", "short", "ushort", "uint", "long", "ulong", "float", "double", "decimal":
        return [.number("0")]
    case "bool":
        return [.keyword("false")]
    case "char":
        return [.string("'\\0'")]
    case "string":
        return [.keyword("null")]
    case "void":
        return []
    default:
        if returnType.hasSuffix("[]") || returnType.hasSuffix("?") || returnType.contains("<") {
            return [.keyword("null")]
        }
        // Non-builtin types: (TypeName)null
        return [.punct("("), .type_(returnType), .punct(")"), .keyword("null")]
    }
}

/// Emit a method name, splitting explicit interface implementations like
/// `IEquatable<IntPtr>.Equals` into clickable type + `.` + method name.
func appendExplicitInterfaceMethod(_ tokens: inout [CodeToken], _ name: String) {
    // Find the last `.` outside of generic brackets
    var depth = 0
    var lastDot: String.Index? = nil
    for idx in name.indices {
        switch name[idx] {
        case "<": depth += 1
        case ">": depth -= 1
        case "." where depth == 0: lastDot = idx
        default: break
        }
    }
    // If there's a dot and it's not the leading `.` of .ctor/.cctor, split
    if let dot = lastDot, dot != name.startIndex {
        let interfacePart = String(name[name.startIndex..<dot])
        let methodPart = String(name[name.index(after: dot)...])
        appendTypeName(&tokens, interfacePart)
        tokens.append(.punct("."))
        tokens.append(.method(methodPart))
    } else {
        tokens.append(.method(name))
    }
}

func appendTypeName(_ tokens: inout [CodeToken], _ typeName: String) {
    parseTypeTokens(typeName, into: &tokens)
}

func appendVisibility(_ tokens: inout [CodeToken], _ vis: Visibility) {
    tokens.append(.keyword(visibilityLabel(vis)))
    tokens.append(.space)
}
