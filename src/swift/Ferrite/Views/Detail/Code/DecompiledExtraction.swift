import SwiftUI

// MARK: - Extract Single Method from Decompiled Type

/// Extract a single method's decompiled body from the full type output, dedented to column 0.
/// Matches by name; for overloads, picks the candidate whose parameter count matches `member`.
func extractMethodFromDecompiled(_ code: String, member: MemberInfo, typeName: String) -> String? {
    let lines = code.components(separatedBy: "\n")
    let isCtor = member.methodAttributes?.isConstructor ?? false
    let isStaticCtor = member.name == ".cctor"

    let searchName: String
    if isStaticCtor {
        searchName = "static \(typeName)("
    } else if isCtor {
        searchName = "\(typeName)("
    } else {
        searchName = "\(member.name)("
    }

    var candidateRanges: [(start: Int, end: Int)] = []

    var i = 0
    while i < lines.count {
        let trimmed = lines[i].trimmingCharacters(in: .whitespaces)

        // Match declaration: contains the search name and "(", not a semicolon-terminated abstract/call.
        if trimmed.contains(searchName) && trimmed.contains("(") &&
           !trimmed.hasPrefix("[") && !trimmed.hasPrefix("//") &&
           !trimmed.contains(";") {
            var declEnd = i
            if !trimmed.contains(")") {
                var d = i + 1
                while d < lines.count {
                    if lines[d].contains(")") { declEnd = d; break }
                    d += 1
                }
            }
            let afterDecl = declEnd + 1
            if afterDecl < lines.count && lines[afterDecl].trimmingCharacters(in: .whitespaces) == "{" {
                if let endIdx = findMatchingBrace(lines: lines, from: afterDecl) {
                    candidateRanges.append((start: i, end: endIdx))
                    i = endIdx + 1
                    continue
                }
            }
            // Also handle { ... } on same line or just declaration + ;
            let lastDeclLine = lines[declEnd].trimmingCharacters(in: .whitespaces)
            if lastDeclLine.hasSuffix("{ ... }") || lastDeclLine.hasSuffix(";") {
                candidateRanges.append((start: i, end: declEnd))
                i = declEnd + 1
                continue
            }
        }
        i += 1
    }

    guard !candidateRanges.isEmpty else { return nil }

    let paramCount = member.parameters.count
    var bestRange = candidateRanges[0]
    for range in candidateRanges {
        var declText = ""
        for lineIdx in range.start...range.end {
            declText += lines[lineIdx]
            if lines[lineIdx].contains(")") { break }
        }
        let extractedParamCount = countParameters(in: declText)
        if extractedParamCount == paramCount {
            bestRange = range
            break
        }
    }

    // Walk backwards to include attribute lines (e.g. [Address(...)]) above the method
    var attrStart = bestRange.start
    while attrStart > 0 {
        let prev = lines[attrStart - 1].trimmingCharacters(in: .whitespaces)
        if prev.hasPrefix("[") && prev.contains("]") {
            attrStart -= 1
        } else {
            break
        }
    }

    let extracted = lines[attrStart...bestRange.end]
    let minIndent = extracted.filter { !$0.trimmingCharacters(in: .whitespaces).isEmpty }
        .map { line in
            var count = 0
            for ch in line { if ch == " " { count += 1 } else { break } }
            return count
        }
        .min() ?? 0

    let dedented = extracted.map { line in
        if line.count >= minIndent {
            return String(line.dropFirst(minIndent))
        }
        return line
    }

    return dedented.joined(separator: "\n")
}

/// Find the matching closing brace for an opening brace at the given line index.
func findMatchingBrace(lines: [String], from start: Int) -> Int? {
    var depth = 0
    for i in start..<lines.count {
        let trimmed = lines[i].trimmingCharacters(in: .whitespaces)
        for ch in trimmed {
            if ch == "{" { depth += 1 }
            else if ch == "}" { depth -= 1 }
        }
        if depth == 0 { return i }
    }
    return nil
}

/// Count the number of parameters in a method declaration line.
func countParameters(in declLine: String) -> Int {
    guard let openParen = declLine.firstIndex(of: "("),
          let closeParen = declLine.lastIndex(of: ")") else { return 0 }
    let paramStr = declLine[declLine.index(after: openParen)..<closeParen]
        .trimmingCharacters(in: .whitespaces)
    if paramStr.isEmpty { return 0 }
    // Count top-level commas, ignoring those inside generic angle brackets (e.g. Dict<K,V>).
    var count = 1
    var depth = 0
    for ch in paramStr {
        if ch == "<" { depth += 1 }
        else if ch == ">" { depth -= 1 }
        else if ch == "," && depth == 0 { count += 1 }
    }
    return count
}

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

