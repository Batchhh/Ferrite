import SwiftUI

// MARK: - Decompiled Property Block Collapsing

/// Collapse multi-line property blocks in decompiled code using PropertyInfo metadata.
func collapseDecompiledPropertyBlocks(
    _ lines: [CodeLine],
    properties: [PropertyInfo],
    expandedProperties: Set<UInt32>
) -> [CodeLine] {
    let propMap = Dictionary(uniqueKeysWithValues: properties.map { ($0.name, $0) })
    var result: [CodeLine] = []
    var i = 0

    while i < lines.count {
        if let prop = matchDecompiledPropertyDecl(lines[i], propMap: propMap),
           i + 1 < lines.count,
           isStandaloneBraceLine(lines[i + 1]) {
            if let endIdx = findMatchingBraceLine(in: lines, from: i + 1) {
                if expandedProperties.contains(prop.token) {
                    // Expanded: keep declaration, make braces clickable, show body
                    result.append(lines[i])
                    let indent = extractLeadingWhitespace(of: lines[i + 1])
                    result.append(CodeLine(tokens: [
                        .plain(indent),
                        CodeToken(text: "{", color: .codeCollapsed, typeName: "ferrite://prop/\(prop.token)")
                    ]))
                    for j in (i + 2)..<endIdx {
                        result.append(lines[j])
                    }
                    result.append(CodeLine(tokens: [
                        .plain(indent),
                        CodeToken(text: "}", color: .codeCollapsed, typeName: "ferrite://prop/\(prop.token)")
                    ]))
                    i = endIdx + 1
                } else {
                    // Collapsed: single line with clickable { get; set; }
                    var t = lines[i].tokens
                    t.append(.space)
                    let link = "ferrite://prop/\(prop.token)"
                    let c: Color = .codeCollapsed
                    t.append(CodeToken(text: "{ ", color: c, typeName: link))
                    if prop.getterToken != nil { t.append(CodeToken(text: "get; ", color: c, typeName: link)) }
                    if prop.setterToken != nil { t.append(CodeToken(text: "set; ", color: c, typeName: link)) }
                    t.append(CodeToken(text: "}", color: c, typeName: link))
                    result.append(CodeLine(tokens: t))
                    i = endIdx + 1
                }
                continue
            }
        }
        result.append(lines[i])
        i += 1
    }

    return result
}

/// Match a tokenized line against known property names.
/// Only matches declaration-like lines (no parens, semicolons, or braces).
func matchDecompiledPropertyDecl(_ line: CodeLine, propMap: [String: PropertyInfo]) -> PropertyInfo? {
    let allText = line.tokens.map(\.text).joined()
    guard !allText.contains("("), !allText.contains(";"), !allText.contains("{") else { return nil }

    guard let last = line.tokens.last(where: {
        !$0.text.trimmingCharacters(in: .whitespaces).isEmpty
    }) else { return nil }

    return propMap[last.text]
}

/// Return `true` if the line contains only a single `{` (possibly with whitespace).
func isStandaloneBraceLine(_ line: CodeLine) -> Bool {
    line.tokens.map(\.text).joined().trimmingCharacters(in: .whitespaces) == "{"
}

/// Find the index of the closing `}` that balances the `{` at `startIdx`.
func findMatchingBraceLine(in lines: [CodeLine], from startIdx: Int) -> Int? {
    var depth = 0
    for i in startIdx..<lines.count {
        let text = lines[i].tokens.map(\.text).joined()
        for ch in text {
            if ch == "{" { depth += 1 }
            if ch == "}" {
                depth -= 1
                if depth == 0 { return i }
            }
        }
    }
    return nil
}

/// Return the leading whitespace string from the first token of a line.
func extractLeadingWhitespace(of line: CodeLine) -> String {
    guard let first = line.tokens.first else { return "" }
    var ws = ""
    for ch in first.text {
        if ch == " " || ch == "\t" { ws.append(ch) } else { break }
    }
    return ws
}
