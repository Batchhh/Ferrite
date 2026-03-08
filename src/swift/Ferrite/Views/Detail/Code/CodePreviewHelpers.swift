import SwiftUI

// MARK: - Decompiled Code Parser

extension CodePreviewView {
    func parseDecompiledCode(_ code: String, type_: TypeInfo? = nil, assemblyId: String? = nil, expandedProperties: Set<UInt32> = [], expandedMethods: Set<UInt32> = []) -> [CodeLine] {
        // Build member name → link URL maps for clickable references
        var memberMethodLinks: [String: String] = [:]
        if let type_ = type_, let assemblyId = assemblyId {
            for member in type_.members {
                let link = "ferrite://member/\(assemblyId)/\(type_.token)/\(member.token)"
                switch member.kind {
                case .method:
                    if memberMethodLinks[member.name] == nil {
                        memberMethodLinks[member.name] = link
                    }
                default:
                    break
                }
            }
        }

        let keywords: Set<String> = [
            "public", "private", "protected", "internal",
            "static", "virtual", "override", "abstract", "sealed",
            "readonly", "const", "volatile",
            "class", "struct", "interface", "enum", "delegate",
            "namespace", "using", "new",
            "null", "true", "false",
            "void", "int", "string", "bool", "float", "double",
            "long", "short", "byte", "char", "decimal",
            "object", "var", "this", "base",
            "get", "set", "is", "as", "typeof", "sizeof",
            "ref", "out", "params",
        ]
        let controlFlow: Set<String> = [
            "if", "else", "while", "for", "foreach", "in",
            "do", "switch", "case", "default",
            "break", "continue", "return",
            "try", "catch", "finally", "throw",
        ]

        var allLines = code.components(separatedBy: "\n").map { line -> CodeLine in
            if line.trimmingCharacters(in: .whitespaces).isEmpty {
                return CodeLine.empty
            }

            var tokens: [CodeToken] = []
            let chars = Array(line)
            var i = 0

            var leadingSpace = ""
            while i < chars.count && (chars[i] == " " || chars[i] == "\t") {
                leadingSpace.append(chars[i])
                i += 1
            }
            if !leadingSpace.isEmpty {
                tokens.append(CodeToken(text: leadingSpace, color: .codePlain))
            }

            while i < chars.count {
                let ch = chars[i]

                // Preprocessor directive
                if ch == "#" && tokens.allSatisfy({ $0.text.trimmingCharacters(in: .whitespaces).isEmpty }) {
                    let rest = String(chars[i...])
                    tokens.append(CodeToken(text: rest, color: .codePreprocessor))
                    break
                }

                if ch == "/" && i + 1 < chars.count && chars[i + 1] == "/" {
                    let rest = String(chars[i...])
                    let isXmlDoc = i + 2 < chars.count && chars[i + 2] == "/"
                    tokens.append(CodeToken(text: rest, color: isXmlDoc ? .codeXmlDoc : .codeComment))
                    break
                }
                if ch == "/" && i + 1 < chars.count && chars[i + 1] == "*" {
                    let rest = String(chars[i...])
                    tokens.append(CodeToken(text: rest, color: .codeComment))
                    break
                }

                if ch == "\"" {
                    var str = "\""
                    i += 1
                    while i < chars.count && chars[i] != "\"" {
                        if chars[i] == "\\" && i + 1 < chars.count {
                            str.append(chars[i])
                            i += 1
                        }
                        str.append(chars[i])
                        i += 1
                    }
                    if i < chars.count { str.append(chars[i]); i += 1 }
                    tokens.append(CodeToken(text: str, color: .codeString))
                    continue
                }

                if ch.isNumber || (ch == "-" && i + 1 < chars.count && chars[i + 1].isNumber) {
                    var num = String(ch)
                    i += 1
                    while i < chars.count && (chars[i].isNumber || chars[i] == "." || chars[i] == "f") {
                        num.append(chars[i])
                        i += 1
                    }
                    tokens.append(CodeToken(text: num, color: .codeNumber))
                    continue
                }

                if ch.isLetter || ch == "_" || ch == "@" {
                    var word = String(ch)
                    i += 1
                    while i < chars.count && (chars[i].isLetter || chars[i].isNumber || chars[i] == "_") {
                        word.append(chars[i])
                        i += 1
                    }
                    if controlFlow.contains(word) {
                        tokens.append(CodeToken(text: word, color: .codeControlFlow))
                    } else if keywords.contains(word) {
                        tokens.append(CodeToken(text: word, color: .codeKeyword))
                    } else {
                        var j = i
                        while j < chars.count && chars[j] == " " { j += 1 }
                        let nextIsParen = j < chars.count && chars[j] == "("
                        let prevIsDot = tokens.last?.text == "."

                        if nextIsParen {
                            if let link = memberMethodLinks[word] {
                                tokens.append(CodeToken(text: word, color: .codeMethod, typeName: link))
                            } else {
                                tokens.append(CodeToken(text: word, color: .codeMethod))
                            }
                        } else if prevIsDot {
                            // After a dot: distinguish qualified type (Ns.Type) from member access (obj.Prop)
                            let tokenBeforeDot = tokens.dropLast().last(where: {
                                !$0.text.trimmingCharacters(in: .whitespaces).isEmpty
                            })
                            let isQualifiedType = tokenBeforeDot?.color == .codeType
                            if word.first?.isUppercase == true && isQualifiedType {
                                // Part of a qualified type name: UnityEngine.MonoBehaviour
                                tokens.append(CodeToken(text: word, color: .codeType, typeName: word))
                            } else if word.first?.isUppercase == true {
                                tokens.append(CodeToken(text: word, color: .codeType))
                            } else {
                                tokens.append(CodeToken(text: word, color: .codeField))
                            }
                        } else if word.first?.isUppercase == true {
                            // PascalCase — decide if it's a declared member name or a type reference.
                            // A member name follows a type token: `float Length;`
                            let prevToken = tokens.last(where: {
                                !$0.text.trimmingCharacters(in: .whitespaces).isEmpty
                            })
                            let prevIsType = prevToken?.color == .codeType
                                || (prevToken?.color == .codeKeyword && isBuiltinType(prevToken?.text ?? ""))
                                || prevToken?.text == ">" || prevToken?.text == "]" || prevToken?.text == "?"
                            // Check if this is an explicit interface prefix: followed by `<` or `.`
                            let nextChar: Character? = i < chars.count ? chars[i] : nil
                            let isInterfacePrefix = nextChar == "<" || nextChar == "."
                            if prevIsType && !isInterfacePrefix {
                                // Follows a type → this is the declared name (field/prop/var)
                                tokens.append(CodeToken(text: word, color: .codeField))
                            } else {
                                tokens.append(CodeToken(text: word, color: .codeType, typeName: word))
                            }
                        } else {
                            tokens.append(CodeToken(text: word, color: .codePlain))
                        }
                    }
                    continue
                }

                tokens.append(CodeToken(text: String(ch), color: .codePlain))
                i += 1
            }

            return CodeLine(tokens: tokens)
        }

        if let type_ = type_, !type_.properties.isEmpty {
            allLines = collapseDecompiledPropertyBlocks(
                allLines, properties: type_.properties, expandedProperties: expandedProperties
            )
        }

        if let type_ = type_ {
            allLines = collapseDecompiledMethodBlocks(
                allLines, members: type_.members, typeName: type_.name, expandedMethods: expandedMethods
            )
        }

        return allLines
    }
}
