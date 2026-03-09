import SwiftUI

// MARK: - Type Code Generation

/// Generate syntax-colored code lines for a full type declaration.
///
/// Hides backing fields, property accessor methods, and compiler-generated nested types.
/// `expandedProperties`/`expandedMethods` control which bodies are shown inline.
func generateTypeCode(_ type_: TypeInfo, expandedProperties: Set<UInt32> = [], expandedMethods: Set<UInt32> = []) -> [CodeLine] {
    let ind = "    "
    var lines: [CodeLine] = []

    for attr in type_.attributesList {
        lines.append(attributeLine(attr, indent: ""))
    }

    var decl: [CodeToken] = []
    appendVisibility(&decl, type_.attributes.visibility)
    if type_.attributes.isStatic {
        decl.append(.keyword("static")); decl.append(.space)
    } else {
        if type_.attributes.isAbstract && type_.kind == .`class` { decl.append(.keyword("abstract")); decl.append(.space) }
        if type_.attributes.isSealed && type_.kind == .`class`  { decl.append(.keyword("sealed"));   decl.append(.space) }
    }
    decl.append(.keyword(typeKindLabel(type_.kind)))
    decl.append(.space)
    decl.append(.type_(type_.name))

    // Base type and interfaces
    var inheritanceList: [String] = []
    if let base = type_.baseType {
        inheritanceList.append(base)
    }
    inheritanceList.append(contentsOf: type_.interfaces)
    if !inheritanceList.isEmpty {
        decl.append(.plain(" : "))
        for (i, item) in inheritanceList.enumerated() {
            if i > 0 { decl.append(.plain(", ")) }
            parseTypeTokens(item, into: &decl)
        }
    }

    lines.append(.init(tokens: decl))
    lines.append(.init(tokens: [.punct("{")]))

    let fields  = type_.members.filter { $0.kind == .field }
    let methods = type_.members.filter { $0.kind == .method }

    if type_.kind == .`enum` {
        // Enum: show members with values, skip the underlying "value__" field
        let enumMembers = fields.filter { f in
            f.fieldAttributes?.isStatic == true && f.fieldAttributes?.isLiteral == true
        }
        for (i, f) in enumMembers.enumerated() {
            var t: [CodeToken] = [.plain(ind), .field(f.name)]
            if let val = f.constantValue {
                t.append(.plain(" = "))
                t.append(.number(val))
            }
            if i < enumMembers.count - 1 {
                t.append(.punct(","))
            }
            lines.append(.init(tokens: t))
        }
    } else {
        // Hide backing fields and property accessor methods
        let propertyAccessorTokens: Set<UInt32> = Set(type_.properties.flatMap { p in
            [p.getterToken, p.setterToken].compactMap { $0 }
        })
        let regularFields = fields.filter { backingFieldPropertyName($0.name) == nil }
        let regularMethods = methods.filter { !propertyAccessorTokens.contains($0.token) }

        for f in regularFields {
            for attr in f.attributesList {
                lines.append(attributeLine(attr, indent: ind))
            }
            lines.append(fieldLine(f, indent: ind))
        }

        if !type_.properties.isEmpty {
            if !regularFields.isEmpty { lines.append(.empty) }
            for (i, prop) in type_.properties.enumerated() {
                for attr in prop.attributesList {
                    lines.append(attributeLine(attr, indent: ind))
                }
                if expandedProperties.contains(prop.token) {
                    lines.append(contentsOf: propertyInfoLines(prop, methods: methods, indent: ind))
                } else {
                    lines.append(collapsedPropertyLine(prop, methods: methods, indent: ind))
                }
                if i < type_.properties.count - 1 { lines.append(.empty) }
            }
        }

        if (!regularFields.isEmpty || !type_.properties.isEmpty) && !regularMethods.isEmpty { lines.append(.empty) }
        for (i, m) in regularMethods.enumerated() {
            for attr in m.attributesList {
                lines.append(attributeLine(attr, indent: ind))
            }
            lines.append(contentsOf: methodLines(m, typeName: type_.name, typeKind: type_.kind, indent: ind, expanded: expandedMethods.contains(m.token)))
            if i < regularMethods.count - 1 { lines.append(.empty) }
        }
    }

    if !type_.nestedTypes.isEmpty {
        lines.append(.empty)
        for nested in type_.nestedTypes {
            lines.append(.init(tokens: [.plain(ind), .comment("// Nested: \(nested.name)")]))
        }
    }

    lines.append(.init(tokens: [.punct("}")]))
    return lines
}

// MARK: - Member Code Generation

/// Generate syntax-colored code lines for a single member declaration.
func generateMemberCode(_ member: MemberInfo, declaringType: TypeInfo) -> [CodeLine] {
    var lines: [CodeLine] = []
    for attr in member.attributesList {
        lines.append(attributeLine(attr, indent: ""))
    }
    switch member.kind {
    case .field:  lines.append(fieldLine(member, indent: ""))
    case .method: lines.append(contentsOf: methodLines(member, typeName: declaringType.name, typeKind: declaringType.kind, indent: ""))
    default:      lines.append(.init(tokens: [.plain(member.name)]))
    }
    return lines
}

// MARK: - Property Code Generation

/// Generate syntax-colored code lines for a standalone property view.
func generatePropertyCode(_ prop: PropertyInfo, declaringType: TypeInfo) -> [CodeLine] {
    var lines: [CodeLine] = []
    for attr in prop.attributesList {
        lines.append(attributeLine(attr, indent: ""))
    }
    let methods = declaringType.members.filter { $0.kind == .method }
    lines.append(contentsOf: propertyInfoLines(prop, methods: methods, indent: ""))
    return lines
}

// MARK: - Event Code Generation

/// Generate syntax-colored code lines for a standalone event view.
func generateEventCode(_ event: EventInfo, declaringType: TypeInfo) -> [CodeLine] {
    var lines: [CodeLine] = []
    for attr in event.attributesList {
        lines.append(attributeLine(attr, indent: ""))
    }

    var decl: [CodeToken] = []
    // Derive visibility from add accessor
    if let addToken = event.addToken,
       let accessor = declaringType.members.first(where: { $0.token == addToken }),
       let attrs = accessor.methodAttributes {
        appendVisibility(&decl, attrs.visibility)
        if attrs.isStatic { decl.append(.keyword("static")); decl.append(.space) }
    }
    decl.append(.keyword("event"))
    decl.append(.space)
    let et = event.eventType.isEmpty ? "object" : event.eventType
    parseTypeTokens(et, into: &decl)
    decl.append(.space)
    decl.append(.field(event.name))
    decl.append(.punct(";"))
    lines.append(.init(tokens: decl))
    return lines
}
