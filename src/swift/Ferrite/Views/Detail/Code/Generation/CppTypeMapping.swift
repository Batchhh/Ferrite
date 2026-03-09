import AppKit

// MARK: - C++ Type Mapping

/// Map a C# type name to a C++ type name.
func cppTypeName(_ csType: String) -> String {
    let name = csType.trimmingCharacters(in: .whitespaces)

    if name.hasSuffix("?") {
        return "std::optional<\(cppTypeName(String(name.dropLast(1))))>"
    }
    if name.hasSuffix("[]") {
        return "std::vector<\(cppTypeName(String(name.dropLast(2))))>"
    }

    if let openIdx = findTopLevelGenericOpen(name) {
        let outer = String(name[name.startIndex..<openIdx])
        let argsContent = String(name[name.index(after: openIdx)..<name.index(before: name.endIndex)])
        let args = splitGenericArgs(argsContent).map { cppTypeName($0.trimmingCharacters(in: .whitespaces)) }
        let joined = args.joined(separator: ", ")

        switch outer {
        case "List", "IList", "ICollection", "IEnumerable", "IReadOnlyList", "IReadOnlyCollection":
            return "std::vector<\(joined)>"
        case "Dictionary", "IDictionary", "IReadOnlyDictionary", "SortedDictionary":
            return "std::unordered_map<\(joined)>"
        case "HashSet", "ISet":
            return "std::unordered_set<\(joined)>"
        case "KeyValuePair":
            return "std::pair<\(joined)>"
        case "Nullable":
            return "std::optional<\(joined)>"
        case "Action":
            return "std::function<void(\(joined))>"
        case "Func":
            if args.count > 1 {
                let ret = args.last!
                let params = args.dropLast().joined(separator: ", ")
                return "std::function<\(ret)(\(params))>"
            }
            return "std::function<\(joined)()>"
        case "Tuple", "ValueTuple":
            return "std::tuple<\(joined)>"
        case "Task", "ValueTask":
            return joined
        default:
            return cppIdentifier(outer)
        }
    }

    switch name {
    case "void":    return "void"
    case "bool":    return "bool"
    case "char":    return "char16_t"
    case "sbyte":   return "int8_t"
    case "byte":    return "uint8_t"
    case "short":   return "int16_t"
    case "ushort":  return "uint16_t"
    case "int":     return "int32_t"
    case "uint":    return "uint32_t"
    case "long":    return "int64_t"
    case "ulong":   return "uint64_t"
    case "float":   return "float"
    case "double":  return "double"
    case "decimal": return "double"
    case "string":  return "std::string"
    case "object":  return "void*"
    case "IntPtr":  return "intptr_t"
    case "UIntPtr": return "uintptr_t"
    case "Type":    return "void*"
    default:        return cppIdentifier(name)
    }
}

// MARK: - Referenced Type Extraction

func extractReferencedTypeNames(_ typeName: String) -> [String] {
    var result: [String] = []
    var name = typeName.trimmingCharacters(in: .whitespaces)

    while name.hasSuffix("[]") { name = String(name.dropLast(2)) }
    while name.hasSuffix("?") { name = String(name.dropLast(1)) }

    if let openIdx = findTopLevelGenericOpen(name) {
        let outer = String(name[name.startIndex..<openIdx])
        if !isBuiltinType(outer) && !outer.isEmpty {
            result.append(stripAritySuffix(outer))
        }
        let argsContent = String(name[name.index(after: openIdx)..<name.index(before: name.endIndex)])
        for arg in splitGenericArgs(argsContent) {
            result.append(contentsOf: extractReferencedTypeNames(arg.trimmingCharacters(in: .whitespaces)))
        }
    } else if !isBuiltinType(name) && !name.isEmpty {
        result.append(stripAritySuffix(name))
    }

    return result
}
