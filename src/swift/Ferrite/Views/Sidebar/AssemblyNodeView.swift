import SwiftUI
import AppKit

// MARK: - Assembly Node

struct AssemblyNode: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @Environment(ItemTagService.self) private var tagService
    let entry: AssemblySummary
    @State private var isHovered = false

    private var nodeId: String { "assembly:\(entry.id)" }
    private var isExpanded: Bool { service.isExpanded(nodeId) }

    private var totalTypeCount: Int {
        entry.namespaces.reduce(0) { $0 + Int($1.typeCount) }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: "externaldrive")
                    .font(.system(size: 11))
                    .foregroundStyle(.tertiary)
                    .frame(width: 16)
                Text(entry.name)
                    .font(.system(size: 13))
                    .foregroundStyle(.primary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Spacer(minLength: 4)
                TagIndicator(tags: tagService.tags(for: .assembly(id: entry.id)))
                CountBadge(count: totalTypeCount)
            }
            .sidebarInteractiveRow(isHovered: $isHovered, isSelected: service.selection == .assembly(id: entry.id))
            .onTapGesture {
                withAnimation(.spring(duration: 0.25, bounce: 0)) {
                    service.toggleExpanded(nodeId)
                }
                service.selection = .assembly(id: entry.id)
            }

            if isExpanded {
                let asmTagged = !tagService.tags(for: .assembly(id: entry.id)).isDisjoint(with: tagService.activeFilters)
                let namespaces = (tagService.isFiltering && !asmTagged)
                    ? entry.namespaces.filter { ns in
                        tagService.shouldShowNamespace(
                            assemblyId: entry.id,
                            namespace: ns.name,
                            typeTokens: service.getNamespaceTypes(assemblyId: entry.id, namespace: ns.name).map(\.token)
                        )
                    }
                    : entry.namespaces
                LazyVStack(spacing: 1) {
                    ForEach(namespaces, id: \.name) { ns in
                        NamespaceNode(assemblyId: entry.id, namespace: ns, depth: 1)
                    }
                }
                .transition(.opacity)
            }
        }
        .contextMenu {
            Menu("Tags") {
                TagContextMenu(selection: .assembly(id: entry.id))
            }
            Divider()
            Button("Export to .cs…") {
                exportAssembly(entry)
            }
            Divider()
            Button("Close Assembly", role: .destructive) {
                projectService.removeAssembly(id: entry.id, filePath: entry.filePath, in: service)
            }
        }
    }

    private func exportAssembly(_ entry: AssemblySummary) {
        var parts: [String] = []
        for ns in entry.namespaces {
            let types = service.getNamespaceTypes(assemblyId: entry.id, namespace: ns.name)
            for typeSummary in types {
                if let typeInfo = service.getTypeDetails(assemblyId: entry.id, token: typeSummary.token) {
                    let lines = generateTypeCode(typeInfo)
                    parts.append(lines.map { $0.tokens.map(\.text).joined() }.joined(separator: "\n"))
                }
            }
        }
        let text = parts.joined(separator: "\n\n")
        let panel = NSSavePanel()
        panel.title = "Export Assembly as C#"
        panel.nameFieldStringValue = "\(entry.name).cs"
        panel.allowedContentTypes = [.init(filenameExtension: "cs")!]
        if panel.runModal() == .OK, let url = panel.url {
            try? text.write(to: url, atomically: true, encoding: .utf8)
        }
    }
}

// MARK: - Namespace Node

struct NamespaceNode: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ItemTagService.self) private var tagService
    let assemblyId: String
    let namespace: NamespaceSummary
    let depth: Int
    @State private var isHovered = false

    private var nodeId: String { "namespace:\(assemblyId):\(namespace.name)" }
    private var isExpanded: Bool { service.isExpanded(nodeId) }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: "folder")
                    .font(.system(size: 11))
                    .foregroundStyle(.tertiary)
                    .frame(width: 16)
                Text(namespace.name.isEmpty ? "(global)" : namespace.name)
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.tail)
                Spacer(minLength: 4)
                TagIndicator(tags: tagService.tags(for: .namespace(assemblyId: assemblyId, name: namespace.name)))
                CountBadge(count: Int(namespace.typeCount))
            }
            .padding(.leading, CGFloat(depth) * 18)
            .sidebarInteractiveRow(isHovered: $isHovered, isSelected: service.selection == .namespace(assemblyId: assemblyId, name: namespace.name))
            .onTapGesture {
                withAnimation(.spring(duration: 0.25, bounce: 0)) {
                    service.toggleExpanded(nodeId)
                }
                service.selection = .namespace(assemblyId: assemblyId, name: namespace.name)
            }

            if isExpanded {
                let types = service.getNamespaceTypes(assemblyId: assemblyId, namespace: namespace.name)
                let nsSelection = Selection.namespace(assemblyId: assemblyId, name: namespace.name)
                let nsTagged = !tagService.tags(for: nsSelection).isDisjoint(with: tagService.activeFilters)
                let visibleTypes = (tagService.isFiltering && !nsTagged)
                    ? types.filter { tagService.shouldShow(.type(assemblyId: assemblyId, token: $0.token)) }
                    : types
                LazyVStack(spacing: 1) {
                    ForEach(visibleTypes, id: \.token) { typeSummary in
                        TypeSummaryRow(assemblyId: assemblyId, typeSummary: typeSummary, depth: depth + 1)
                    }
                }
                .transition(.opacity)
            }
        }
        .contextMenu {
            Menu("Tags") {
                TagContextMenu(selection: .namespace(assemblyId: assemblyId, name: namespace.name))
            }
        }
    }
}
