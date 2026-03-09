import SwiftUI
import AppKit

// MARK: - Type Summary Row (sidebar — uses lightweight TypeSummary, fetches details on expand)

struct TypeSummaryRow: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ItemTagService.self) private var tagService
    let assemblyId: String
    let typeSummary: TypeSummary
    let depth: Int
    @State private var isHovered = false

    private var nodeId: String { "type:\(assemblyId):\(typeSummary.token)" }
    private var isExpanded: Bool { service.isExpanded(nodeId) }

    private var totalCount: Int {
        Int(typeSummary.memberCount + typeSummary.propertyCount)
    }

    private var hasChildren: Bool {
        typeSummary.memberCount > 0 || typeSummary.propertyCount > 0 || typeSummary.nestedTypeCount > 0
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: iconForTypeKind(typeSummary.kind))
                    .font(.system(size: 11))
                    .foregroundStyle(.tertiary)
                    .frame(width: 16)
                Text(typeSummary.name)
                    .font(.system(size: 13))
                    .lineLimit(1)
                    .truncationMode(.tail)
                Spacer(minLength: 4)
                TagIndicator(tags: tagService.tags(for: .type(assemblyId: assemblyId, token: typeSummary.token)))
                if totalCount > 0 {
                    CountBadge(count: totalCount)
                }
            }
            .padding(.leading, CGFloat(depth) * 18)
            .sidebarInteractiveRow(isHovered: $isHovered, isSelected: service.selection == .type(assemblyId: assemblyId, token: typeSummary.token))
            .onTapGesture {
                if hasChildren {
                    withAnimation(.spring(duration: 0.25, bounce: 0)) {
                        service.toggleExpanded(nodeId)
                    }
                }
                service.selection = .type(assemblyId: assemblyId, token: typeSummary.token)
            }

            if hasChildren && isExpanded, let typeInfo = service.getTypeDetails(assemblyId: assemblyId, token: typeSummary.token) {
                LazyVStack(spacing: 1) {
                    ForEach(typeInfo.nestedTypes, id: \.token) { nested in
                        TypeRow(assemblyId: assemblyId, type_: nested, depth: depth + 1)
                    }
                    ForEach(typeInfo.properties, id: \.token) { prop in
                        PropertyNode(assemblyId: assemblyId, typeToken: typeSummary.token, property: prop, type_: typeInfo, depth: depth + 1)
                    }
                    ForEach(nonPropertyMembers(typeInfo), id: \.token) { member in
                        MemberRow(assemblyId: assemblyId, typeToken: typeSummary.token, member: member, depth: depth + 1)
                    }
                }
                .transition(.opacity)
            }
        }
        .contextMenu {
            Menu("Tags") {
                TagContextMenu(selection: .type(assemblyId: assemblyId, token: typeSummary.token))
            }
            Divider()
            Button("Copy C# Code") {
                if let typeInfo = service.getTypeDetails(assemblyId: assemblyId, token: typeSummary.token) {
                    copyTypeCode(typeInfo)
                }
            }
            Button("Export to .cs…") {
                if let typeInfo = service.getTypeDetails(assemblyId: assemblyId, token: typeSummary.token) {
                    exportTypeCode(typeInfo)
                }
            }
            Button("Export to .h…") {
                if let typeInfo = service.getTypeDetails(assemblyId: assemblyId, token: typeSummary.token) {
                    exportHeaderCode(typeInfo)
                }
            }
        }
    }

    private func nonPropertyMembers(_ typeInfo: TypeInfo) -> [MemberInfo] {
        let propAccessorTokens: Set<UInt32> = Set(typeInfo.properties.flatMap { prop in
            [prop.getterToken, prop.setterToken].compactMap { $0 }
        })
        return typeInfo.members.filter { !propAccessorTokens.contains($0.token) }
    }

    private func codeText(_ type_: TypeInfo) -> String {
        generateTypeCode(type_).map { $0.tokens.map(\.text).joined() }.joined(separator: "\n")
    }

    private func copyTypeCode(_ type_: TypeInfo) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(codeText(type_), forType: .string)
    }

    private func exportTypeCode(_ type_: TypeInfo) {
        let panel = NSSavePanel()
        panel.title = "Export C# Preview"
        panel.nameFieldStringValue = "\(type_.name).cs"
        panel.allowedContentTypes = [.init(filenameExtension: "cs")!]
        if panel.runModal() == .OK, let url = panel.url {
            try? codeText(type_).write(to: url, atomically: true, encoding: .utf8)
        }
    }

    private func exportHeaderCode(_ type_: TypeInfo) {
        let text = generateHeaderExport(rootType: type_, assemblyId: assemblyId, service: service)
        let panel = NSSavePanel()
        panel.title = "Export Fields-Only Header"
        panel.nameFieldStringValue = "\(type_.name).h"
        panel.allowedContentTypes = [.init(filenameExtension: "h")!]
        if panel.runModal() == .OK, let url = panel.url {
            try? text.write(to: url, atomically: true, encoding: .utf8)
        }
    }
}
