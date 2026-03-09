import SwiftUI
import AppKit

// Requires macOS 26 (Tahoe) for Liquid Glass APIs.
struct AssemblyTreeView: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @Environment(SearchService.self) private var searchService
    @Environment(ItemTagService.self) private var tagService
    @State private var showFilterPopover = false

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 6) {
                Text("Assemblies")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(.tertiary)
                    .textCase(.uppercase)
                    .tracking(0.5)
                Spacer()
                Button {
                    searchService.isPresented = true
                } label: {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundStyle(.tertiary)
                        .frame(width: 22, height: 22)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .onHover { hovering in
                    if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
                }
                Button {
                    showFilterPopover = true
                } label: {
                    Image(systemName: tagService.isFiltering ? "line.3.horizontal.decrease.circle.fill" : "line.3.horizontal.decrease.circle")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundStyle(tagService.isFiltering ? .primary : .tertiary)
                        .frame(width: 22, height: 22)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .onHover { hovering in
                    if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
                }
                .popover(isPresented: $showFilterPopover, arrowEdge: .bottom) {
                    TagFilterPopover()
                }
                Button {
                    projectService.showOpenPanel(in: service)
                } label: {
                    Image(systemName: "plus")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundStyle(.tertiary)
                        .frame(width: 22, height: 22)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .onHover { hovering in
                    if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
                }
            }
            .padding(.horizontal, 14)
            .padding(.top, 10)
            .padding(.bottom, 6)

            Group {
                if service.isLoading && service.loadedAssemblies.isEmpty {
                    VStack(spacing: 10) {
                        ProgressView()
                        Text("Loading\u{2026}")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if service.loadedAssemblies.isEmpty {
                    EmptyAssembliesView()
                } else {
                    assemblyList
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            if let project = projectService.currentProject {
                ProjectSwitcherBar(project: project)
            }
        }
    }

    private var assemblyList: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 2) {
                let filtered = service.loadedAssemblies.filter { entry in
                    tagService.shouldShow(.assembly(id: entry.id))
                }
                ForEach(filtered, id: \.id) { entry in
                    AssemblyNode(entry: entry)
                }
            }
            .padding(.horizontal, 2)
            .padding(.vertical, 2)
        }
        .scrollIndicators(.never)
        .overlay(alignment: .bottom) {
            if service.isLoading {
                HStack(spacing: 6) {
                    ProgressView().scaleEffect(0.75)
                    Text("Loading\u{2026}")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
                .modifier(GlassEffectModifier(shape: .capsule))
                .padding(.bottom, 16)
                .transition(.move(edge: .bottom).combined(with: .opacity))
                .animation(.spring(duration: 0.3), value: service.isLoading)
            }
        }
    }
}

// MARK: - Empty State

private struct EmptyAssembliesView: View {
    var body: some View {
        Text("No Assemblies")
            .font(.callout)
            .foregroundStyle(.tertiary)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
