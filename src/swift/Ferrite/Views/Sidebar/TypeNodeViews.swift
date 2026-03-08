import SwiftUI

// MARK: - Type Row (for nested types inside expanded TypeSummaryRow — already has full TypeInfo)

struct TypeRow: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ItemTagService.self) private var tagService
    let assemblyId: String
    let type_: TypeInfo
    let depth: Int
    @State private var isHovered = false

    private var nodeId: String { "type:\(assemblyId):\(type_.token)" }
    private var isExpanded: Bool { service.isExpanded(nodeId) }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: iconForTypeKind(type_.kind))
                    .font(.system(size: 11))
                    .foregroundStyle(.tertiary)
                    .frame(width: 16)
                Text(type_.name)
                    .font(.system(size: 13))
                    .lineLimit(1)
                    .truncationMode(.tail)
                Spacer(minLength: 4)
                TagIndicator(tags: tagService.tags(for: .type(assemblyId: assemblyId, token: type_.token)))
                if !type_.members.isEmpty || !type_.properties.isEmpty {
                    CountBadge(count: type_.members.count + type_.properties.count)
                }
            }
            .padding(.leading, CGFloat(depth) * 18)
            .sidebarInteractiveRow(isHovered: $isHovered, isSelected: service.selection == .type(assemblyId: assemblyId, token: type_.token))
            .onTapGesture {
                if hasChildren {
                    withAnimation(.spring(duration: 0.25, bounce: 0)) {
                        service.toggleExpanded(nodeId)
                    }
                }
                service.selection = .type(assemblyId: assemblyId, token: type_.token)
            }
            .contextMenu {
                Menu("Tags") {
                    TagContextMenu(selection: .type(assemblyId: assemblyId, token: type_.token))
                }
            }

            if hasChildren && isExpanded {
                LazyVStack(spacing: 1) {
                    ForEach(type_.nestedTypes, id: \.token) { nested in
                        TypeRow(assemblyId: assemblyId, type_: nested, depth: depth + 1)
                    }
                    ForEach(type_.properties, id: \.token) { prop in
                        PropertyNode(assemblyId: assemblyId, typeToken: type_.token, property: prop, type_: type_, depth: depth + 1)
                    }
                    ForEach(nonPropertyMembers, id: \.token) { member in
                        MemberRow(assemblyId: assemblyId, typeToken: type_.token, member: member, depth: depth + 1)
                    }
                }
                .transition(.opacity)
            }
        }
    }

    private var hasChildren: Bool {
        !type_.nestedTypes.isEmpty || !type_.members.isEmpty || !type_.properties.isEmpty
    }

    private var nonPropertyMembers: [MemberInfo] {
        let propAccessorTokens: Set<UInt32> = Set(type_.properties.flatMap { prop in
            [prop.getterToken, prop.setterToken].compactMap { $0 }
        })
        return type_.members.filter { !propAccessorTokens.contains($0.token) }
    }
}

// MARK: - Member Row

struct MemberRow: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ItemTagService.self) private var tagService
    let assemblyId: String
    let typeToken: UInt32
    let member: MemberInfo
    let depth: Int
    @State private var isHovered = false

    var body: some View {
        HStack(spacing: 8) {
            memberIcon
                .frame(width: 16)
            Text(member.name)
                .font(.system(size: 13))
                .lineLimit(1)
                .truncationMode(.tail)
            Spacer(minLength: 4)
            TagIndicator(tags: tagService.tags(for: .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: member.token)))
        }
        .padding(.leading, CGFloat(depth) * 18)
        .sidebarInteractiveRow(isHovered: $isHovered, isSelected: service.selection == .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: member.token))
        .onTapGesture {
            service.selection = .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: member.token)
        }
        .contextMenu {
            Menu("Tags") {
                TagContextMenu(selection: .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: member.token))
            }
        }
    }

    private var memberIcon: some View {
        Image(systemName: iconForMemberKind(member.kind))
            .font(.system(size: 11))
            .foregroundStyle(.tertiary)
    }
}

// MARK: - Property Node

struct PropertyNode: View {
    @Environment(DecompilerService.self) private var service
    let assemblyId: String
    let typeToken: UInt32
    let property: PropertyInfo
    let type_: TypeInfo
    let depth: Int
    @State private var isHovered = false

    private var nodeId: String { "prop:\(assemblyId):\(typeToken):\(property.token)" }
    private var isExpanded: Bool { service.isExpanded(nodeId) }

    private var accessors: [MemberInfo] {
        let tokens = [property.getterToken, property.setterToken].compactMap { $0 }
        return type_.members.filter { tokens.contains($0.token) }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: "p.square")
                    .font(.system(size: 11))
                    .foregroundStyle(.tertiary)
                    .frame(width: 16)
                Text(property.name)
                    .font(.system(size: 13))
                    .lineLimit(1)
                    .truncationMode(.tail)
                Spacer(minLength: 4)
            }
            .padding(.leading, CGFloat(depth) * 18)
            .sidebarInteractiveRow(isHovered: $isHovered, isSelected: false)
            .onTapGesture {
                if !accessors.isEmpty {
                    withAnimation(.spring(duration: 0.25, bounce: 0)) {
                        service.toggleExpanded(nodeId)
                    }
                }
                service.selection = .type(assemblyId: assemblyId, token: typeToken)
            }

            if isExpanded {
                VStack(spacing: 1) {
                    ForEach(accessors, id: \.token) { accessor in
                        MemberRow(assemblyId: assemblyId, typeToken: typeToken, member: accessor, depth: depth + 1)
                    }
                }
                .transition(.opacity)
            }
        }
    }
}
