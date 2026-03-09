import SwiftUI

// MARK: - Breadcrumb Bar

struct BreadcrumbBar: View {
    @Environment(DecompilerService.self) private var service

    var body: some View {
        HStack(spacing: 0) {
            ForEach(Array(crumbs.enumerated()), id: \.offset) { index, crumb in
                if index > 0 {
                    Image(systemName: "chevron.right")
                        .font(.system(size: 8, weight: .bold))
                        .foregroundStyle(.quaternary)
                        .padding(.horizontal, 2)
                }
                BreadcrumbSegment(crumb: crumb, isLast: index == crumbs.count - 1) {
                    service.selection = crumb.selection
                }
            }
            WindowDragArea()
        }
        .padding(.leading, 14)
    }

    private var crumbs: [Crumb] {
        guard let selection = service.selection else { return [] }
        var result: [Crumb] = []

        switch selection {
        case .type(let assemblyId, let token):
            if let assembly = service.findAssembly(id: assemblyId) {
                result.append(Crumb(
                    label: assembly.name,
                    icon: "shippingbox",
                    color: .secondary,
                    selection: .assembly(id: assemblyId)
                ))
                if let type_ = service.findType(assemblyId: assemblyId, token: token) {
                    let ns = type_.namespace
                    result.append(Crumb(
                        label: ns.isEmpty ? "(global)" : ns,
                        icon: "folder",
                        color: .secondary,
                        selection: .namespace(assemblyId: assemblyId, name: ns)
                    ))
                    result.append(Crumb(
                        label: type_.name,
                        icon: iconForTypeKind(type_.kind),
                        color: colorForTypeKind(type_.kind),
                        selection: .type(assemblyId: assemblyId, token: token)
                    ))
                }
            }

        case .member(let assemblyId, let typeToken, let memberToken):
            if let assembly = service.findAssembly(id: assemblyId) {
                result.append(Crumb(
                    label: assembly.name,
                    icon: "shippingbox",
                    color: .secondary,
                    selection: .assembly(id: assemblyId)
                ))
                if let type_ = service.findType(assemblyId: assemblyId, token: typeToken) {
                    let ns = type_.namespace
                    result.append(Crumb(
                        label: ns.isEmpty ? "(global)" : ns,
                        icon: "folder",
                        color: .secondary,
                        selection: .namespace(assemblyId: assemblyId, name: ns)
                    ))
                    result.append(Crumb(
                        label: type_.name,
                        icon: iconForTypeKind(type_.kind),
                        color: colorForTypeKind(type_.kind),
                        selection: .type(assemblyId: assemblyId, token: typeToken)
                    ))
                    if let member = service.findMember(assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken) {
                        result.append(Crumb(
                            label: member.name,
                            icon: iconForMemberKind(member.kind),
                            color: colorForMemberKind(member.kind),
                            selection: .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken)
                        ))
                    } else if let prop = service.findProperty(assemblyId: assemblyId, typeToken: typeToken, propertyToken: memberToken) {
                        result.append(Crumb(
                            label: prop.name,
                            icon: iconForMemberKind(.property),
                            color: colorForMemberKind(.property),
                            selection: .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken)
                        ))
                    } else if let event = service.findEvent(assemblyId: assemblyId, typeToken: typeToken, eventToken: memberToken) {
                        result.append(Crumb(
                            label: event.name,
                            icon: iconForMemberKind(.event),
                            color: colorForMemberKind(.event),
                            selection: .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken)
                        ))
                    }
                }
            }

        default:
            break
        }

        return result
    }
}

struct Crumb {
    let label: String
    let icon: String
    let color: Color
    let selection: Selection
}

struct BreadcrumbSegment: View {
    let crumb: Crumb
    let isLast: Bool
    let action: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: crumb.icon)
                    .font(.system(size: 9, weight: .medium))
                    .foregroundStyle(.secondary)
                Text(crumb.label)
                    .font(.system(size: 11, weight: isLast ? .semibold : .regular))
                    .foregroundStyle(isLast ? .primary : .secondary)
                    .lineLimit(1)
            }
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(
                RoundedRectangle(cornerRadius: 5)
                    .fill(.primary.opacity(isHovered ? 0.08 : 0))
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
            if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
        }
        .animation(.easeInOut(duration: 0.12), value: isHovered)
    }
}
