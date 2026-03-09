import AppKit

// MARK: - Single Type Generation

private func cppConstant(_ value: String, cppType: String) -> String {
    if value == "null" { return "nullptr" }
    if value == "true" || value == "false" { return value }
    if cppType == "float" && !value.hasSuffix("f") { return value + "f" }
    return value
}

private func isArithmeticCppType(_ t: String) -> Bool {
    switch t {
    case "bool", "char16_t",
         "int8_t", "uint8_t", "int16_t", "uint16_t",
         "int32_t", "uint32_t", "int64_t", "uint64_t",
         "float", "double",
         "intptr_t", "uintptr_t":
        return true
    default:
        return false
    }
}

/// Extract FieldOffset value from a member's attributes, e.g. "0x10".
private func fieldOffset(_ member: MemberInfo) -> String? {
    for attr in member.attributesList {
        if attr.name == "FieldOffset" {
            for arg in attr.arguments {
                // Format: `Offset = "0x10"` or just `"0x10"`
                let cleaned = arg.replacingOccurrences(of: "Offset = ", with: "")
                    .trimmingCharacters(in: CharacterSet(charactersIn: "\""))
                if cleaned.hasPrefix("0x") || cleaned.first?.isNumber == true {
                    return cleaned
                }
            }
        }
    }
    return nil
}

func generateFieldsOnlyType(_ type_: TypeInfo) -> String {
    let safeName = cppIdentifier(type_.name)
    var lines: [String] = []

    if type_.kind == .`enum` {
        let fields = type_.members.filter { $0.kind == .field }
        let underlyingField = fields.first { $0.name == "value__" }
        let underlying = underlyingField.map { cppTypeName($0.fieldType) } ?? "int32_t"

        lines.append("enum class \(safeName) : \(underlying) {")
        let enumMembers = fields.filter { f in
            f.fieldAttributes?.isStatic == true && f.fieldAttributes?.isLiteral == true
        }
        for (i, f) in enumMembers.enumerated() {
            var line = "    \(cppIdentifier(f.name))"
            if let val = f.constantValue {
                line += " = \(val)"
            }
            if i < enumMembers.count - 1 { line += "," }
            lines.append(line)
        }
        lines.append("};")
        return lines.joined(separator: "\n")
    }

    if type_.kind == .delegate {
        let invoke = type_.members.first { $0.kind == .method && $0.name == "Invoke" }
        if let invoke = invoke {
            let ret = cppTypeName(invoke.returnType.isEmpty ? "void" : invoke.returnType)
            let params = invoke.parameters.map { cppTypeName($0.typeName) }.joined(separator: ", ")
            lines.append("using \(safeName) = std::function<\(ret)(\(params))>;")
        } else {
            lines.append("using \(safeName) = std::function<void()>;")
        }
        return lines.joined(separator: "\n")
    }

    if type_.kind == .interface {
        lines.append("struct \(safeName) {")
        lines.append("    virtual ~\(safeName)() = default;")
        lines.append("};")
        return lines.joined(separator: "\n")
    }

    // Class or struct → C++ struct, with inheritance from non-BCL types
    var decl = "struct \(safeName)"
    var bases: [String] = []
    if let base = type_.baseType {
        let baseName = stripAritySuffix(base)
        if !isBuiltinType(baseName) && !bclSkipNames.contains(baseName) {
            let mapped = cppTypeName(base)
            if !bclSkipNames.contains(mapped) {
                bases.append("public \(mapped)")
            }
        }
    }
    for iface in type_.interfaces {
        let ifaceName = stripAritySuffix(iface)
        // Strip generic args to get plain name for BCL check
        let plainName: String
        if let openIdx = findTopLevelGenericOpen(ifaceName) {
            plainName = String(ifaceName[ifaceName.startIndex..<openIdx])
        } else {
            plainName = ifaceName
        }
        if !bclSkipNames.contains(plainName) && !bclGenericNames.contains(plainName) {
            bases.append("public \(cppIdentifier(ifaceName))")
        }
    }
    if !bases.isEmpty {
        decl += " : " + bases.joined(separator: ", ")
    }
    lines.append(decl + " {")

    let fields = type_.members.filter { $0.kind == .field }
    let regularFields = fields.filter { backingFieldPropertyName($0.name) == nil }

    for f in regularFields {
        let ft = f.fieldType.isEmpty ? "object" : f.fieldType
        let cppType = cppTypeName(ft)
        let offsetComment = fieldOffset(f).map { " // \($0)" } ?? ""

        var line = "    "
        if let a = f.fieldAttributes {
            if a.isStatic && a.isLiteral {
                if isArithmeticCppType(cppType) {
                    line += "static constexpr \(cppType) \(f.name)"
                } else {
                    line += "static inline const \(cppType) \(f.name)"
                }
                if let val = f.constantValue {
                    line += " = \(cppConstant(val, cppType: cppType))"
                }
                line += ";\(offsetComment)"
            } else if a.isStatic {
                line += "static "
                if a.isInitOnly { line += "const " }
                line += "\(cppType) \(f.name);\(offsetComment)"
            } else {
                if a.isInitOnly { line += "const " }
                line += "\(cppType) \(f.name);\(offsetComment)"
            }
        } else {
            line += "\(cppType) \(f.name);\(offsetComment)"
        }

        lines.append(line)
    }

    lines.append("};")
    return lines.joined(separator: "\n")
}
