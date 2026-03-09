import SwiftUI

// MARK: - Search Bar

struct SearchBarView: View {
    @Binding var query: String
    let hasActiveFilters: Bool
    let showFilters: Bool
    let activeFilterCount: Int
    let hasTags: Bool
    let onReturn: () -> Void
    let onMoveUp: () -> Void
    let onMoveDown: () -> Void
    let onDeleteSelected: () -> Void
    let onEscape: () -> Void
    let onToggleFilters: () -> Void
    let onNewProject: () -> Void
    @FocusState.Binding var isFieldFocused: Bool

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "folder")
                .font(.system(size: 18, weight: .light))
                .foregroundStyle(.secondary)

            TextField("Search projects\u{2026}", text: $query)
                .textFieldStyle(.plain)
                .font(.system(size: 18, weight: .light))
                .focused($isFieldFocused)
                .onKeyPress(.return) {
                    onReturn()
                    return .handled
                }
                .onKeyPress(.upArrow) {
                    onMoveUp()
                    return .handled
                }
                .onKeyPress(.downArrow) {
                    onMoveDown()
                    return .handled
                }
                .onKeyPress(keys: [.delete], phases: .down) { press in
                    guard press.modifiers.contains(.command) else { return .ignored }
                    onDeleteSelected()
                    return .handled
                }
                .onKeyPress(.escape) {
                    onEscape()
                    return .handled
                }

            if !query.isEmpty {
                Button {
                    withAnimation(.easeOut(duration: 0.15)) {
                        query = ""
                    }
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 14))
                        .foregroundStyle(.quaternary)
                }
                .buttonStyle(.plain)
            }

            if hasTags {
                Button {
                    onToggleFilters()
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "line.3.horizontal.decrease")
                            .font(.system(size: 10, weight: .medium))
                        if hasActiveFilters {
                            Text("\(activeFilterCount)")
                                .font(.system(size: 10, weight: .bold, design: .rounded))
                        }
                    }
                    .foregroundStyle(hasActiveFilters || showFilters ? .primary : .quaternary)
                    .frame(height: 22)
                    .padding(.horizontal, 6)
                    .background(RoundedRectangle(cornerRadius: 5)
                        .fill(.white.opacity(hasActiveFilters || showFilters ? 0.08 : 0.06)))
                    .overlay(RoundedRectangle(cornerRadius: 5)
                        .strokeBorder(.white.opacity(hasActiveFilters || showFilters ? 0.12 : 0.08), lineWidth: 0.5))
                }
                .buttonStyle(.plain)
            }

            Button {
                onNewProject()
            } label: {
                Image(systemName: "plus")
                    .font(.system(size: 9, weight: .semibold))
                    .foregroundStyle(.quaternary)
                    .frame(width: 22, height: 22)
                    .background(
                        RoundedRectangle(cornerRadius: 5)
                            .fill(.white.opacity(0.06))
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 5)
                            .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
                    )
            }
            .buttonStyle(.plain)

            if query.isEmpty {
                shortcutHint("\u{2318}P")
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 14)
    }
}

// MARK: - Filter Popover Tag Row

private struct FilterPopoverTagRow: View {
    let tag: ProjectTag
    let isActive: Bool
    let onToggle: () -> Void

    var body: some View {
        Button { onToggle() } label: {
            HStack(spacing: 6) {
                Circle().fill(tag.color.color).frame(width: 8, height: 8)
                Text(tag.name)
                    .font(.system(size: 11, weight: isActive ? .semibold : .regular))
                    .foregroundStyle(isActive ? AnyShapeStyle(.primary) : AnyShapeStyle(.secondary))
                Spacer()
                if isActive {
                    Image(systemName: "checkmark")
                        .font(.system(size: 9, weight: .bold))
                        .foregroundStyle(.secondary)
                }
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 5)
            .background(RoundedRectangle(cornerRadius: 5).fill(.white.opacity(isActive ? 0.08 : 0)))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Filter Popover

struct FilterPopoverView: View {
    let availableTags: [ProjectTag]
    let activeTagFilters: Set<UUID>
    let onToggleTag: (UUID) -> Void
    let onClearFilters: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ForEach(availableTags) { tag in
                FilterPopoverTagRow(
                    tag: tag,
                    isActive: activeTagFilters.contains(tag.id),
                    onToggle: { onToggleTag(tag.id) }
                )
            }

            if !availableTags.isEmpty {
                Divider().opacity(0.2).padding(.horizontal, 4)
                Button { onClearFilters() } label: {
                    Text("Clear filters")
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(.tertiary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(8)
        .frame(width: 180)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(.white.opacity(0.08), lineWidth: 0.5))
        .shadow(color: .black.opacity(0.3), radius: 20, y: 8)
    }
}

