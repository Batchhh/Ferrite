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


