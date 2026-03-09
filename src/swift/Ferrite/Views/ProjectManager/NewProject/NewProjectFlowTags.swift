import SwiftUI

extension NewProjectSheet {
    // MARK: - Tags Section

    var newTagForm: some View {
        VStack(spacing: 0) {
            if !projectService.availableTags.isEmpty {
                HStack(spacing: 4) {
                    ForEach(projectService.availableTags) { tag in
                        NewProjectTagChip(
                            tag: tag,
                            isSelected: selectedTags.contains(tag.id)
                        ) {
                            withAnimation(.easeOut(duration: 0.15)) {
                                if selectedTags.contains(tag.id) {
                                    _ = selectedTags.remove(tag.id)
                                } else {
                                    selectedTags.insert(tag.id)
                                }
                            }
                        } onDelete: {
                            _ = selectedTags.remove(tag.id)
                            projectService.deleteTag(id: tag.id)
                        }
                    }
                    Spacer()
                }
                .padding(.horizontal, 18)
                .padding(.vertical, 8)

                Divider().opacity(0.3)
            }

            if showingNewTag {
                HStack(spacing: 12) {
                    TextField("Tag name\u{2026}", text: $newTagName)
                        .textFieldStyle(.plain)
                        .font(.system(size: 14, weight: .light))
                        .focused($tagNameFocused)
                        .onSubmit { addTag() }

                    HStack(spacing: 2) {
                        ForEach(TagColor.allCases, id: \.self) { tagColor in
                            Button {
                                newTagColor = tagColor
                            } label: {
                                Circle()
                                    .fill(tagColor.color)
                                    .frame(width: 10, height: 10)
                                    .opacity(newTagColor == tagColor ? 1 : 0.35)
                                    .scaleEffect(newTagColor == tagColor ? 1.2 : 1)
                            }
                            .buttonStyle(.plain)
                            .animation(.easeInOut(duration: 0.1), value: newTagColor)
                        }
                    }
                }
                .padding(.horizontal, 18)
                .padding(.vertical, 10)
                .transition(.opacity.combined(with: .blurReplace))
            }
        }
    }
}

// MARK: - New Project Tag Chip

struct NewProjectTagChip: View {
    let tag: ProjectTag
    let isSelected: Bool
    let onToggle: () -> Void
    let onDelete: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: onToggle) {
            HStack(spacing: 4) {
                Circle()
                    .fill(isSelected ? tag.color.color : tag.color.color.opacity(0.5))
                    .frame(width: 6, height: 6)
                Text(tag.name)
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(isSelected ? .primary : .tertiary)
                    .lineLimit(1)
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 5)
            .background(
                RoundedRectangle(cornerRadius: 6)
                    .fill(.white.opacity(isSelected ? 0.08 : isHovered ? 0.04 : 0))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 6)
                    .strokeBorder(.white.opacity(isSelected ? 0.1 : 0), lineWidth: 0.5)
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { isHovered = $0 }
        .animation(.easeOut(duration: 0.1), value: isHovered)
        .animation(.easeOut(duration: 0.1), value: isSelected)
        .contextMenu {
            Button(role: .destructive, action: onDelete) {
                Label("Delete Tag", systemImage: "trash")
            }
        }
    }
}
