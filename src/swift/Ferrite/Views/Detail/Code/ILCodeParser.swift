import SwiftUI

// MARK: - IL Code Parser

extension CodePreviewView {
    func parseILCode(_ code: String) -> [CodeLine] {
        let directives: Set<String> = [
            ".class", ".method", ".field", ".maxstack", ".locals",
            ".entrypoint", ".try", ".catch", ".finally", ".override",
            ".property", ".event", ".custom", ".pack", ".size",
        ]
        let ilKeywords: Set<String> = [
            "cil", "managed", "public", "private", "family", "assembly",
            "famorassem", "famandassem", "privatescope",
            "static", "instance", "virtual", "abstract", "sealed",
            "hidebysig", "specialname", "rtspecialname", "newslot",
            "extends", "implements", "init", "nested", "interface",
            "auto", "ansi", "beforefieldinit",
            "to", "handler", "catch", "finally", "fault", "filter",
        ]

        return code.components(separatedBy: "\n").map { line in
            if line.trimmingCharacters(in: .whitespaces).isEmpty {
                return CodeLine.empty
            }

            var tokens: [CodeToken] = []
            let chars = Array(line)
            var i = 0

            // Leading whitespace
            var leading = ""
            while i < chars.count && (chars[i] == " " || chars[i] == "\t") {
                leading.append(chars[i])
                i += 1
            }
            if !leading.isEmpty {
                tokens.append(CodeToken(text: leading, color: .codePlain))
            }

            while i < chars.count {
                let ch = chars[i]

                // Comments
                if ch == "/" && i + 1 < chars.count && chars[i + 1] == "/" {
                    tokens.append(CodeToken(text: String(chars[i...]), color: .codeComment))
                    break
                }

                // String literals
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

                // IL label: IL_XXXX:
                if ch == "I" && i + 2 < chars.count && chars[i + 1] == "L" && chars[i + 2] == "_" {
                    var label = ""
                    let start = i
                    while i < chars.count && chars[i] != " " && chars[i] != ":" && chars[i] != "," && chars[i] != ")" {
                        label.append(chars[i])
                        i += 1
                    }
                    if i < chars.count && chars[i] == ":" {
                        label.append(":")
                        i += 1
                        tokens.append(CodeToken(text: label, color: .codeNumber))
                    } else {
                        // IL label as branch target (no colon)
                        tokens.append(CodeToken(text: label, color: .codeNumber))
                    }
                    _ = start
                    continue
                }

                // Directives starting with .
                if ch == "." {
                    var word = "."
                    i += 1
                    while i < chars.count && (chars[i].isLetter || chars[i].isNumber || chars[i] == "_") {
                        word.append(chars[i])
                        i += 1
                    }
                    if directives.contains(word) {
                        tokens.append(CodeToken(text: word, color: .codeKeyword))
                    } else {
                        // Opcode with dot (e.g. ldarg.0, conv.i4)
                        // Append further dotted parts
                        while i < chars.count && chars[i] == "." {
                            word.append(chars[i])
                            i += 1
                            while i < chars.count && (chars[i].isLetter || chars[i].isNumber || chars[i] == "_") {
                                word.append(chars[i])
                                i += 1
                            }
                        }
                        tokens.append(CodeToken(text: word, color: .codeControlFlow))
                    }
                    continue
                }

                // Numbers
                if ch.isNumber || (ch == "-" && i + 1 < chars.count && chars[i + 1].isNumber) {
                    var num = String(ch)
                    i += 1
                    while i < chars.count && (chars[i].isNumber || chars[i] == "." || chars[i] == "x" || chars[i].isHexDigit) {
                        num.append(chars[i])
                        i += 1
                    }
                    tokens.append(CodeToken(text: num, color: .codeNumber))
                    continue
                }

                // Words
                if ch.isLetter || ch == "_" {
                    var word = String(ch)
                    i += 1
                    while i < chars.count && (chars[i].isLetter || chars[i].isNumber || chars[i] == "_") {
                        word.append(chars[i])
                        i += 1
                    }

                    // Check for dotted opcode (e.g. ldarg.0)
                    while i < chars.count && chars[i] == "." {
                        word.append(chars[i])
                        i += 1
                        while i < chars.count && (chars[i].isLetter || chars[i].isNumber || chars[i] == "_") {
                            word.append(chars[i])
                            i += 1
                        }
                    }

                    if ilKeywords.contains(word) {
                        tokens.append(CodeToken(text: word, color: .codeKeyword))
                    } else if isILOpcode(word) {
                        tokens.append(CodeToken(text: word, color: .codeControlFlow))
                    } else if word.first?.isUppercase == true {
                        let shortName = word.components(separatedBy: ".").last ?? word
                        tokens.append(CodeToken(text: word, color: .codeType, typeName: shortName))
                    } else {
                        tokens.append(CodeToken(text: word, color: .codePlain))
                    }
                    continue
                }

                // Colons (:: for member refs)
                if ch == ":" && i + 1 < chars.count && chars[i + 1] == ":" {
                    tokens.append(CodeToken(text: "::", color: .codePlain))
                    i += 2
                    continue
                }

                tokens.append(CodeToken(text: String(ch), color: .codePlain))
                i += 1
            }

            return CodeLine(tokens: tokens)
        }
    }

    private func isILOpcode(_ word: String) -> Bool {
        let opcodes: Set<String> = [
            "nop", "break", "ret", "dup", "pop", "throw", "rethrow",
            "ldarg", "ldarga", "starg", "ldloc", "ldloca", "stloc",
            "ldnull", "ldstr", "ldtoken",
            "ldc", "ldfld", "ldflda", "stfld", "ldsfld", "ldsflda", "stsfld",
            "ldobj", "stobj", "cpobj",
            "ldlen", "ldelema", "ldelem", "stelem", "newarr",
            "call", "callvirt", "calli", "newobj", "jmp",
            "br", "brfalse", "brtrue", "beq", "bne", "bge", "bgt", "ble", "blt",
            "switch", "leave", "endfinally", "endfilter",
            "add", "sub", "mul", "div", "rem", "neg", "not",
            "and", "or", "xor", "shl", "shr",
            "ceq", "cgt", "clt",
            "conv", "castclass", "isinst", "box", "unbox",
            "sizeof", "localloc", "ckfinite",
            "arglist", "mkrefany", "refanyval", "refanytype",
            "cpblk", "initblk", "initobj",
            "ldftn", "ldvirtftn", "constrained", "volatile", "tail",
            "readonly", "unaligned",
        ]
        let base = word.split(separator: ".").first.map(String.init) ?? word
        return opcodes.contains(base)
    }
}
