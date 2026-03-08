import SwiftUI

// MARK: - Namespace Detail

struct NamespaceDetailView: View {
    let namespaceName: String
    let types: [TypeSummary]
    let assemblyName: String

    private var displayName: String {
        namespaceName.isEmpty ? "(global)" : namespaceName
    }

    private var kindCounts: [(kind: TypeKind, count: Int)] {
        var map: [TypeKind: Int] = [:]
        for type_ in types {
            map[type_.kind, default: 0] += 1
        }
        // Stable order: class, struct, interface, enum, delegate
        let order: [TypeKind] = [.`class`, .`struct`, .interface, .`enum`, .delegate]
        return order.compactMap { kind in
            guard let count = map[kind] else { return nil }
            return (kind, count)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            Spacer()

            VStack(spacing: 28) {
                VStack(spacing: 12) {
                    Image(systemName: "folder")
                        .font(.system(size: 32, weight: .thin))
                        .foregroundStyle(.tertiary)
                        .padding(.bottom, 2)

                    Text(displayName)
                        .font(.system(size: 20, weight: .semibold))
                        .textSelection(.enabled)

                    HStack(spacing: 4) {
                        Image(systemName: "shippingbox")
                            .font(.system(size: 9))
                        Text(assemblyName)
                    }
                    .font(.system(.caption2, design: .monospaced))
                    .foregroundStyle(.quaternary)
                }

                if !kindCounts.isEmpty {
                    HStack(spacing: 14) {
                        ForEach(kindCounts, id: \.kind) { item in
                            HStack(spacing: 5) {
                                Image(systemName: iconForTypeKind(item.kind))
                                    .font(.system(size: 10))
                                    .foregroundStyle(.tertiary)
                                Text("\(item.count) \(typeKindLabel(item.kind))\(item.count == 1 ? "" : "s")")
                                    .font(.caption2)
                                    .foregroundStyle(.quaternary)
                            }
                        }
                    }
                }
            }

            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Member Detail

struct MemberDetailView: View {
    let member: MemberInfo
    let declaringType: TypeInfo

    private var visibility: String? {
        if let v = member.fieldAttributes?.visibility { return visibilityLabel(v) }
        if let v = member.methodAttributes?.visibility { return visibilityLabel(v) }
        return nil
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                memberHeader
                    .padding(16)

                if let fa = member.fieldAttributes, fa.isStatic || fa.isLiteral || fa.isInitOnly {
                    Divider().padding(.horizontal, 16)
                    SectionHeader(title: "Modifiers")
                        .padding(.horizontal, 16)
                    if fa.isStatic   { ModifierRow(name: "static").padding(.horizontal, 16) }
                    if fa.isLiteral  { ModifierRow(name: "const").padding(.horizontal, 16) }
                    if fa.isInitOnly { ModifierRow(name: "readonly").padding(.horizontal, 16) }
                }

                if let ma = member.methodAttributes,
                   ma.isConstructor || ma.isStatic || ma.isAbstract || ma.isVirtual || ma.isFinal {
                    Divider().padding(.horizontal, 16)
                    SectionHeader(title: "Modifiers")
                        .padding(.horizontal, 16)
                    if ma.isConstructor { ModifierRow(name: "constructor").padding(.horizontal, 16) }
                    if ma.isStatic      { ModifierRow(name: "static").padding(.horizontal, 16) }
                    if ma.isAbstract    { ModifierRow(name: "abstract").padding(.horizontal, 16) }
                    if ma.isVirtual     { ModifierRow(name: "virtual").padding(.horizontal, 16) }
                    if ma.isFinal       { ModifierRow(name: "final / sealed").padding(.horizontal, 16) }
                }

                if member.kind == .method, !member.returnType.isEmpty {
                    Divider().padding(.horizontal, 16)
                    SectionHeader(title: "Signature")
                        .padding(.horizontal, 16)
                    HStack(spacing: 8) {
                        Text("Returns")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .frame(width: 50, alignment: .trailing)
                        Text(member.returnType)
                            .font(.system(.callout, design: .monospaced))
                            .foregroundStyle(.primary)
                        Spacer()
                    }
                    .padding(.vertical, 4)
                    .padding(.horizontal, 16)

                    if !member.parameters.isEmpty {
                        ForEach(Array(member.parameters.enumerated()), id: \.offset) { _, param in
                            HStack(spacing: 8) {
                                Text("Param")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .frame(width: 50, alignment: .trailing)
                                Text("\(param.typeName) \(param.name)")
                                    .font(.system(.callout, design: .monospaced))
                                    .foregroundStyle(.primary)
                                Spacer()
                            }
                            .padding(.vertical, 2)
                            .padding(.horizontal, 16)
                        }
                    }
                }

                if member.kind == .field, !member.fieldType.isEmpty {
                    Divider().padding(.horizontal, 16)
                    SectionHeader(title: "Type")
                        .padding(.horizontal, 16)
                    HStack(spacing: 8) {
                        Text(member.fieldType)
                            .font(.system(.callout, design: .monospaced))
                            .foregroundStyle(.primary)
                        Spacer()
                    }
                    .padding(.vertical, 4)
                    .padding(.horizontal, 16)
                }

                Divider().padding(.horizontal, 16)
                SectionHeader(title: "Declaring Type")
                    .padding(.horizontal, 16)
                HStack(spacing: 8) {
                    Image(systemName: iconForTypeKind(declaringType.kind))
                        .foregroundStyle(colorForTypeKind(declaringType.kind))
                        .frame(width: 16)
                    Text(declaringType.fullName)
                        .font(.system(.callout, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    Spacer()
                }
                .padding(.vertical, 6)
                .padding(.horizontal, 16)
                .padding(.bottom, 16)
            }
        }
        .scrollIndicators(.hidden)
    }

    @ViewBuilder
    private var memberHeader: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: iconForMemberKind(member.kind))
                    .font(.title2)
                    .foregroundStyle(colorForMemberKind(member.kind))
                    .frame(width: 28, alignment: .center)
                VStack(alignment: .leading, spacing: 3) {
                    Text(member.name)
                        .font(.title3.weight(.semibold))
                        .textSelection(.enabled)
                    Text("\(declaringType.fullName).\(member.name)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                        .lineLimit(3)
                }
                Spacer(minLength: 4)
            }
            Divider()
            HStack(spacing: 6) {
                KindPill(text: memberKindLabel(member.kind), color: colorForMemberKind(member.kind))
                if let vis = visibility { AccessPill(text: vis) }
                Spacer()
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .modifier(GlassEffectModifier(shape: .rect(cornerRadius: 12)))
    }
}
