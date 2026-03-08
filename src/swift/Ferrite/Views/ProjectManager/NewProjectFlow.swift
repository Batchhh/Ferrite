import SwiftUI
import AppKit

// MARK: - New Project Sheet

struct NewProjectSheet: View {
    @Environment(ProjectService.self) private var projectService
    @State private var name = ""
    @State private var selectedTags: Set<UUID> = []
    @State private var showingNewTag = false
    @State private var newTagName = ""
    @State private var newTagColor: TagColor = .blue
    @State private var escMonitor: Any?
    @FocusState private var nameFocused: Bool
    @FocusState private var tagNameFocused: Bool
    var onDismiss: (() -> Void)? = nil
    let onCreate: (String, [UUID]) -> Void

    private var isValid: Bool {
        !name.trimmingCharacters(in: .whitespaces).isEmpty
    }

    var body: some View {
        VStack(spacing: 0) {
            nameField

            if showingNewTag || !projectService.availableTags.isEmpty {
                Divider().opacity(0.3)
                newTagForm
            }
        }
        .frame(width: 480)
        .fixedSize(horizontal: false, vertical: true)
        .onAppear {
            focusNameField()
            escMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
                guard event.keyCode == 53 else { return event }
                close()
                return nil
            }
        }
        .onDisappear {
            if let monitor = escMonitor {
                NSEvent.removeMonitor(monitor)
                escMonitor = nil
            }
        }
    }

    // MARK: - Name Field

    private var nameField: some View {
        HStack(spacing: 12) {
            Image(systemName: "folder.badge.plus")
                .font(.system(size: 18, weight: .light))
                .foregroundStyle(.secondary)

            TextField("New project\u{2026}", text: $name)
                .textFieldStyle(.plain)
                .font(.system(size: 18, weight: .light))
                .focused($nameFocused)
                .onSubmit { create() }

            Button {
                withAnimation(.spring(duration: 0.2, bounce: 0)) {
                    showingNewTag.toggle()
                    if showingNewTag { tagNameFocused = true }
                }
            } label: {
                Image(systemName: "plus")
                    .font(.system(size: 9, weight: .semibold))
                    .foregroundStyle(showingNewTag ? .primary : .quaternary)
                    .frame(width: 22, height: 22)
                    .background(
                        RoundedRectangle(cornerRadius: 5)
                            .fill(.white.opacity(showingNewTag ? 0.08 : 0.06))
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 5)
                            .strokeBorder(.white.opacity(showingNewTag ? 0.12 : 0.08), lineWidth: 0.5)
                    )
            }
            .buttonStyle(.plain)

            shortcutHint("\u{2318}N")
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 14)
    }

    // MARK: - Tags Section

    private var newTagForm: some View {
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

    // MARK: - Actions

    private func addTag() {
        let trimmed = newTagName.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        let tag = projectService.createTag(name: trimmed, color: newTagColor)
        selectedTags.insert(tag.id)
        newTagName = ""
        tagNameFocused = true
    }

    private func close() {
        onDismiss?()
    }

    private func create() {
        let trimmed = name.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        onCreate(trimmed, Array(selectedTags))
        close()
    }

    private func focusNameField() {
        DispatchQueue.main.async {
            nameFocused = true
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            nameFocused = true
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
