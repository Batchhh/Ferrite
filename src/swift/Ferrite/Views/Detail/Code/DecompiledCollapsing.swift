import SwiftUI

// MARK: - Decompiled Method Block Collapsing

/// Collapse multi-line method bodies in decompiled code into clickable `{ ... }` stubs.
func collapseDecompiledMethodBlocks(
    _ lines: [CodeLine],
    members: [MemberInfo],
    typeName: String,
    expandedMethods: Set<UInt32>
) -> [CodeLine] {
    // Map method name → ordered token queue; overloads are matched sequentially.
    var methodTokenQueues: [String: [UInt32]] = [:]
    for member in members where member.kind == .method {
        methodTokenQueues[member.name, default: []].append(member.token)
    }
    // Decompiled output uses the type name for constructors, not ".ctor"/".cctor".
    if let ctorTokens = methodTokenQueues[".ctor"] {
        methodTokenQueues[typeName, default: []].append(contentsOf: ctorTokens)
    }
    if let cctorTokens = methodTokenQueues[".cctor"] {
        methodTokenQueues[typeName, default: []].append(contentsOf: cctorTokens)
    }
    var tokenCursors: [String: Int] = [:]

    var result: [CodeLine] = []
    var i = 0

    while i < lines.count {
        if let match = matchDecompiledMethodDeclStart(lines[i]),
           let tokens = methodTokenQueues[match.name] {
            let declEnd: Int
            if match.complete {
                declEnd = i
            } else {
                // Multi-line parameter list: scan forward for the closing ")".
                var d = i + 1
                while d < lines.count {
                    let text = lines[d].tokens.map(\.text).joined()
                    if text.contains(")") { break }
                    d += 1
                }
                declEnd = d < lines.count ? d : i
            }

            let braceLineIdx = declEnd + 1
            if braceLineIdx < lines.count,
               isStandaloneBraceLine(lines[braceLineIdx]) {
                let cursor = tokenCursors[match.name, default: 0]
                if cursor < tokens.count,
                   let endIdx = findMatchingBraceLine(in: lines, from: braceLineIdx) {
                    let token = tokens[cursor]
                    tokenCursors[match.name] = cursor + 1
                    let link = "ferrite://method/\(token)"
                    let indent = extractLeadingWhitespace(of: lines[braceLineIdx])

                    if expandedMethods.contains(token) {
                        for d in i...declEnd {
                            result.append(lines[d])
                        }
                        result.append(CodeLine(tokens: [
                            .plain(indent),
                            CodeToken(text: "{", color: .codeCollapsed, typeName: link)
                        ]))
                        for j in (braceLineIdx + 1)..<endIdx {
                            result.append(lines[j])
                        }
                        result.append(CodeLine(tokens: [
                            .plain(indent),
                            CodeToken(text: "}", color: .codeCollapsed, typeName: link)
                        ]))
                    } else {
                        for d in i..<declEnd {
                            result.append(lines[d])
                        }
                        var t = lines[declEnd].tokens
                        t.append(.space)
                        let c: Color = .codeCollapsed
                        t.append(CodeToken(text: "{ ... }", color: c, typeName: link))
                        result.append(CodeLine(tokens: t))
                    }
                    i = endIdx + 1
                    continue
                }
            }
        }
        result.append(lines[i])
        i += 1
    }

    return result
}

/// Match result for a method declaration start line.
struct MethodDeclMatch {
    let name: String
    /// Whether the closing ")" is on the same line (single-line params).
    let complete: Bool
}

/// Match a tokenized line as the start of a method declaration.
/// Requires `(` with no `{` or `;`; rejects attribute lines starting with `[`.
func matchDecompiledMethodDeclStart(_ line: CodeLine) -> MethodDeclMatch? {
    let allText = line.tokens.map(\.text).joined()
    let trimmed = allText.trimmingCharacters(in: .whitespaces)
    guard allText.contains("("),
          !allText.contains("{"), !allText.contains(";"),
          !trimmed.hasPrefix("[") else { return nil }

    // Walk backwards from `(` to reconstruct the full method name, including explicit interface prefix.
    // e.g. ["ISerializable", ".", "GetObjectData", "("] → "ISerializable.GetObjectData"
    for (idx, token) in line.tokens.enumerated() {
        if token.text == "(" && idx > 0 {
            var nameParts: [String] = []
            var k = idx - 1
            let nameToken = line.tokens[k]
            let name = nameToken.text.trimmingCharacters(in: .whitespaces)
            guard !name.isEmpty else { continue }
            nameParts.insert(name, at: 0)
            k -= 1
            while k >= 0 {
                let t = line.tokens[k].text
                if t == "." {
                    nameParts.insert(".", at: 0)
                    k -= 1
                    // Before the dot there may be a generic closing `>` — collect the full type expression.
                    if k >= 0 && line.tokens[k].text == ">" {
                        var genericParts: [String] = []
                        var depth = 0
                        while k >= 0 {
                            let gt = line.tokens[k].text
                            if gt == ">" { depth += 1 }
                            else if gt == "<" { depth -= 1 }
                            genericParts.insert(gt, at: 0)
                            k -= 1
                            if depth == 0 { break }
                        }
                        if k >= 0 {
                            let prev = line.tokens[k].text.trimmingCharacters(in: .whitespaces)
                            if !prev.isEmpty && (prev.first?.isLetter == true || prev.first == "_") {
                                genericParts.insert(prev, at: 0)
                                k -= 1
                            }
                        }
                        nameParts.insert(genericParts.joined(), at: 0)
                    } else if k >= 0 {
                        let prev = line.tokens[k].text.trimmingCharacters(in: .whitespaces)
                        if !prev.isEmpty && prev != "." && (prev.first?.isLetter == true || prev.first == "_") {
                            nameParts.insert(prev, at: 0)
                            k -= 1
                        }
                    }
                } else {
                    break
                }
            }
            let fullName = nameParts.joined()
            return MethodDeclMatch(name: fullName, complete: allText.contains(")"))
        }
    }
    return nil
}
