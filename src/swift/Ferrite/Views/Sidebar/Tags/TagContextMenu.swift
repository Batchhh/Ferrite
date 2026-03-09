import SwiftUI

/// Reusable tag submenu content for sidebar context menus.
struct TagContextMenu: View {
    @Environment(ItemTagService.self) private var tagService
    @Environment(ProjectService.self) private var projectService
    let selection: Selection

    var body: some View {
        let currentTags = tagService.tags(for: selection)
        ForEach(ItemTag.allCases, id: \.self) { tag in
            Toggle(isOn: Binding(
                get: { currentTags.contains(tag) },
                set: { _ in
                    tagService.toggleTag(tag, on: selection)
                    tagService.save(to: projectService)
                }
            )) {
                Label(tag.displayName, systemImage: tag.icon)
            }
        }
        if !currentTags.isEmpty {
            Divider()
            Button(role: .destructive) {
                for tag in currentTags {
                    tagService.toggleTag(tag, on: selection)
                }
                tagService.save(to: projectService)
            } label: {
                Label("Remove All Tags", systemImage: "xmark.circle")
            }
        }
    }
}
