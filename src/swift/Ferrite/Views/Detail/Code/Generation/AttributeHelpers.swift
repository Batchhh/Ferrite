import SwiftUI

// MARK: - Attribute Line

func attributeLine(_ attr: AttributeInfo, indent: String) -> CodeLine {
    var t: [CodeToken] = []
    if !indent.isEmpty { t.append(.plain(indent)) }
    t.append(.punct("["))
    t.append(.type_(attr.name))
    if !attr.arguments.isEmpty {
        t.append(.punct("("))
        for (i, arg) in attr.arguments.enumerated() {
            if i > 0 { t.append(.punct(", ")) }
            appendAttributeArg(&t, arg)
        }
        t.append(.punct(")"))
    }
    t.append(.punct("]"))
    return .init(tokens: t)
}

/// Parse a single attribute argument like `RVA = "0x213E058"` or `"0x10"` or `true`
/// and append correctly-colored tokens.
func appendAttributeArg(_ tokens: inout [CodeToken], _ arg: String) {
    // Named argument: `Name = value`
    if let eqRange = arg.range(of: " = ") {
        let name = String(arg[arg.startIndex..<eqRange.lowerBound])
        let value = String(arg[eqRange.upperBound...])
        tokens.append(.plain(name))
        tokens.append(.punct(" = "))
        appendAttributeValue(&tokens, value)
    } else {
        appendAttributeValue(&tokens, arg)
    }
}

/// Append a single attribute argument value with appropriate syntax coloring.
func appendAttributeValue(_ tokens: inout [CodeToken], _ value: String) {
    if value.hasPrefix("\"") && value.hasSuffix("\"") {
        tokens.append(.string(value))
    } else if value == "true" || value == "false" {
        tokens.append(.keyword(value))
    } else if value.first?.isNumber == true || value.hasPrefix("0x") || value.hasPrefix("-") {
        tokens.append(.number(value))
    } else {
        tokens.append(.plain(value))
    }
}

// MARK: - Backing Field Helpers

/// Extract the property name from a compiler-generated backing field name.
/// `<CommonTags>k__BackingField` → `CommonTags`, otherwise nil.
func backingFieldPropertyName(_ fieldName: String) -> String? {
    guard fieldName.hasPrefix("<"),
          let endIdx = fieldName.firstIndex(of: ">"),
          fieldName[fieldName.index(after: endIdx)...].hasPrefix("k__BackingField") else {
        return nil
    }
    return String(fieldName[fieldName.index(after: fieldName.startIndex)..<endIdx])
}

/// Render a property with expanded getter/setter bodies from PropertyInfo metadata.
func propertyInfoLines(_ prop: PropertyInfo, methods: [MemberInfo], indent: String) -> [CodeLine] {
    var lines: [CodeLine] = []
    let innerInd = indent + "    "

    var decl: [CodeToken] = []
    if !indent.isEmpty { decl.append(.plain(indent)) }

    // Derive visibility/modifiers from getter (or setter)
    let accessorToken = prop.getterToken ?? prop.setterToken
    if let token = accessorToken,
       let accessor = methods.first(where: { $0.token == token }),
       let attrs = accessor.methodAttributes {
        appendVisibility(&decl, attrs.visibility)
        if attrs.isStatic { decl.append(.keyword("static")); decl.append(.space) }
        if attrs.isAbstract { decl.append(.keyword("abstract")); decl.append(.space) }
        else if attrs.isVirtual && !attrs.isFinal { decl.append(.keyword("virtual")); decl.append(.space) }
    }

    let pt = prop.propertyType.isEmpty ? "object" : prop.propertyType
    parseTypeTokens(pt, into: &decl)
    decl.append(.space)
    decl.append(.field(prop.name))
    decl.append(.plain(" "))
    decl.append(CodeToken(text: "{", color: .codeCollapsed, typeName: "ferrite://prop/\(prop.token)"))
    lines.append(.init(tokens: decl))

    if let getterToken = prop.getterToken,
       let getter = methods.first(where: { $0.token == getterToken }) {
        for attr in getter.attributesList {
            lines.append(attributeLine(attr, indent: innerInd))
        }
        lines.append(.init(tokens: [.plain(innerInd), .keyword("get")]))
        let returnType = prop.propertyType.isEmpty ? "object" : prop.propertyType
        if getter.methodAttributes?.isAbstract == true {
            lines[lines.count - 1] = .init(tokens: [.plain(innerInd), .keyword("get"), .punct(";")])
        } else {
            lines.append(contentsOf: methodBodyLines(returnType: returnType, indent: innerInd))
        }
    }

    if let setterToken = prop.setterToken,
       let setter = methods.first(where: { $0.token == setterToken }) {
        for attr in setter.attributesList {
            lines.append(attributeLine(attr, indent: innerInd))
        }
        lines.append(.init(tokens: [.plain(innerInd), .keyword("set")]))
        if setter.methodAttributes?.isAbstract == true {
            lines[lines.count - 1] = .init(tokens: [.plain(innerInd), .keyword("set"), .punct(";")])
        } else {
            lines.append(contentsOf: methodBodyLines(returnType: "void", indent: innerInd))
        }
    }

    lines.append(.init(tokens: [
        .plain(indent),
        CodeToken(text: "}", color: .codeCollapsed, typeName: "ferrite://prop/\(prop.token)")
    ]))
    return lines
}

/// Render a collapsed property: `public string Name { ... }` where `...` is clickable.
func collapsedPropertyLine(_ prop: PropertyInfo, methods: [MemberInfo], indent: String) -> CodeLine {
    var t: [CodeToken] = []
    if !indent.isEmpty { t.append(.plain(indent)) }

    let accessorToken = prop.getterToken ?? prop.setterToken
    if let token = accessorToken,
       let accessor = methods.first(where: { $0.token == token }),
       let attrs = accessor.methodAttributes {
        appendVisibility(&t, attrs.visibility)
        if attrs.isStatic { t.append(.keyword("static")); t.append(.space) }
        if attrs.isAbstract { t.append(.keyword("abstract")); t.append(.space) }
        else if attrs.isVirtual && !attrs.isFinal { t.append(.keyword("virtual")); t.append(.space) }
    }

    let pt = prop.propertyType.isEmpty ? "object" : prop.propertyType
    parseTypeTokens(pt, into: &t)
    t.append(.space)
    t.append(.field(prop.name))
    t.append(.plain(" "))
    let propLink = "ferrite://prop/\(prop.token)"
    let c: Color = .codeCollapsed
    t.append(CodeToken(text: "{ ", color: c, typeName: propLink))
    if prop.getterToken != nil { t.append(CodeToken(text: "get; ", color: c, typeName: propLink)) }
    if prop.setterToken != nil { t.append(CodeToken(text: "set; ", color: c, typeName: propLink)) }
    t.append(CodeToken(text: "}", color: c, typeName: propLink))
    return .init(tokens: t)
}

/// Render an auto-property line from a backing field (fallback for standalone member view).
func propertyLine(_ propName: String, field: MemberInfo, hasGetter: Bool, hasSetter: Bool, indent: String) -> CodeLine {
    var t: [CodeToken] = []
    if !indent.isEmpty { t.append(.plain(indent)) }
    if let a = field.fieldAttributes {
        appendVisibility(&t, a.visibility)
    }
    let ft = field.fieldType.isEmpty ? "object" : field.fieldType
    parseTypeTokens(ft, into: &t)
    t.append(.space)
    t.append(.field(propName))
    t.append(.plain(" "))
    t.append(.punct("{"))
    t.append(.plain(" "))
    if hasGetter { t.append(.keyword("get")); t.append(.punct(";")); t.append(.plain(" ")) }
    if hasSetter { t.append(.keyword("set")); t.append(.punct(";")); t.append(.plain(" ")) }
    t.append(.punct("}"))
    return .init(tokens: t)
}
