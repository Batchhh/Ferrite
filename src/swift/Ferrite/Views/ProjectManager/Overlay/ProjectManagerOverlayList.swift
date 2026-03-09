import SwiftUI

// MARK: - Active Filter Chip

struct ActiveFilterChip: View {
    let tag: ProjectTag
    let onRemove: () -> Void

    var body: some View {
        Button { onRemove() } label: {
            HStack(spacing: 4) {
                Circle().fill(tag.color.color).frame(width: 6, height: 6)
                Text(tag.name).font(.system(size: 10, weight: .medium)).foregroundStyle(.secondary)
                Image(systemName: "xmark").font(.system(size: 7, weight: .bold)).foregroundStyle(.tertiary)
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(RoundedRectangle(cornerRadius: 6).fill(.white.opacity(0.06)))
            .overlay(RoundedRectangle(cornerRadius: 6).strokeBorder(.white.opacity(0.08), lineWidth: 0.5))
        }
        .buttonStyle(.plain)
        .transition(.opacity.combined(with: .scale(scale: 0.8)))
    }
}

// MARK: - Active Filter Chips

struct ActiveFilterChipsView: View {
    let availableTags: [ProjectTag]
    let activeTagFilters: Set<UUID>
    let onToggleTag: (UUID) -> Void
    let onClearFilters: () -> Void

    var body: some View {
        HStack(spacing: 6) {
            let activeTags = availableTags.filter { activeTagFilters.contains($0.id) }
            ForEach(activeTags) { tag in
                ActiveFilterChip(tag: tag, onRemove: { onToggleTag(tag.id) })
            }

            if activeTagFilters.count >= 2 {
                Button {
                    onClearFilters()
                } label: {
                    HStack(spacing: 2) {
                        Image(systemName: "xmark")
                            .font(.system(size: 7, weight: .bold))
                        Text("Clear all")
                            .font(.system(size: 10, weight: .medium))
                    }
                    .foregroundStyle(.tertiary)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .transition(.opacity.combined(with: .move(edge: .top)))
    }
}

// MARK: - Project List

struct ProjectListView: View {
    let filteredProjects: [Project]
    let query: String
    let selectedIndex: Int
    let currentProjectId: UUID?
    let onDelete: (Project) -> Void
    let onOpen: (Project) -> Void

    var body: some View {
        Group {
            if filteredProjects.isEmpty && !query.isEmpty {
                HStack(spacing: 8) {
                    Image(systemName: "folder")
                        .font(.system(size: 12))
                        .foregroundStyle(.quaternary)
                    Text("No projects matching \"\(query)\"")
                        .font(.system(size: 13))
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 20)
                .transition(.opacity)
            } else {
                ScrollViewReader { proxy in
                    ScrollView(.vertical, showsIndicators: false) {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(Array(filteredProjects.enumerated()), id: \.element.id) { index, project in
                                ProjectManagerRow(
                                    project: project,
                                    isCurrent: currentProjectId == project.id,
                                    isSelected: index == selectedIndex,
                                    onDelete: { onDelete(project) }
                                )
                                .id(project.id)
                                .onTapGesture { onOpen(project) }
                            }
                        }
                        .padding(.vertical, 6)
                        .padding(.horizontal, 6)
                    }
                    .frame(maxHeight: 340)
                    .onChange(of: selectedIndex) { _, newValue in
                        guard newValue < filteredProjects.count else { return }
                        withAnimation(.easeOut(duration: 0.1)) {
                            proxy.scrollTo(filteredProjects[newValue].id, anchor: .center)
                        }
                    }
                }
            }
        }
    }
}
