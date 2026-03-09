import SwiftUI
import AppKit

// MARK: - New Project Sheet

struct NewProjectSheet: View {
    @Environment(ProjectService.self) var projectService
    @State var name = ""
    @State var selectedTags: Set<UUID> = []
    @State var showingNewTag = false
    @State var newTagName = ""
    @State var newTagColor: TagColor = .blue
    @State private var escMonitor: Any?
    @FocusState private var nameFocused: Bool
    @FocusState var tagNameFocused: Bool
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

    // MARK: - Actions

    func addTag() {
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

