import SwiftUI

// MARK: - Line Builder Helpers

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
