import SwiftUI
import AppKit

// MARK: - Welcome View

struct WelcomeView: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @State private var isDropTargeted = false

    private var recentProjects: [Project] {
        Array(projectService.projects.prefix(5))
    }

    var body: some View {
        ZStack {
            Color(red: 25/255, green: 25/255, blue: 28/255)
                .ignoresSafeArea()

            VStack(spacing: 0) {
                Spacer()
                centerContent
                Spacer()
                footerHint
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onDrop(of: [.fileURL], isTargeted: $isDropTargeted) { providers in
            handleDrop(providers)
        }
        .overlay {
            if isDropTargeted { dropOverlay }
        }
        .animation(.easeInOut(duration: 0.15), value: isDropTargeted)
    }

    private var centerContent: some View {
        VStack(spacing: 28) {
            HStack(alignment: .firstTextBaseline, spacing: 0) {
                Text("ferrite")
                    .font(.system(size: 32, weight: .thin, design: .default))
                    .tracking(6)
                    .foregroundStyle(.white.opacity(0.7))

                Text(" \(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.1.0")")
                    .font(.system(size: 11, weight: .light, design: .monospaced))
                    .foregroundStyle(.white.opacity(0.15))
            }

            VStack(spacing: 2) {
                WelcomeLink(icon: "doc.badge.plus", label: "Open Assembly", shortcut: "⌘O") {
                    projectService.showOpenPanel(in: service)
                }
                WelcomeLink(icon: "plus.rectangle.on.folder", label: "New Project", shortcut: "⌘N") {
                    projectService.showingNewProject = true
                }
                WelcomeLink(icon: "folder", label: "Browse Projects", shortcut: "⌘P") {
                    projectService.showingProjectManager = true
                }
            }

            if !recentProjects.isEmpty {
                VStack(spacing: 8) {
                    Text("Recent")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(.white.opacity(0.2))
                        .tracking(0.8)
                        .textCase(.uppercase)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 10)

                    VStack(spacing: 0) {
                        ForEach(recentProjects) { project in
                            WelcomeProjectRow(project: project) {
                                projectService.openProject(project, in: service)
                            } onDelete: {
                                projectService.deleteProject(project, in: service)
                            }
                        }
                    }
                }
            }
        }
        .frame(width: 280)
    }

    private var footerHint: some View {
        HStack(spacing: 6) {
            Image(systemName: "arrow.down.doc")
                .font(.system(size: 10))
            Text("Drop .dll or .exe to open")
                .font(.system(size: 11))
        }
        .foregroundStyle(.white.opacity(0.15))
        .padding(.bottom, 20)
    }

    private var dropOverlay: some View {
        DropZoneCard()
            .transition(.opacity.combined(with: .scale(scale: 0.92)))
    }

    private func handleDrop(_ providers: [NSItemProvider]) -> Bool {
        let service = service
        let projectService = projectService
        for provider in providers {
            provider.loadItem(forTypeIdentifier: "public.file-url", options: nil) { item, _ in
                guard let data = item as? Data,
                      let url = URL(dataRepresentation: data, relativeTo: nil) else { return }
                let ext = url.pathExtension.lowercased()
                if ext == "dll" || ext == "exe" {
                    Task { @MainActor in
                        projectService.addAssembly(url: url, in: service)
                    }
                }
            }
        }
        return !providers.isEmpty
    }
}

// MARK: - Welcome Link

struct WelcomeLink: View {
    let icon: String
    let label: String
    let shortcut: String
    let action: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                Image(systemName: icon)
                    .font(.system(size: 12))
                    .foregroundColor(isHovered ? .primary : Color.white.opacity(0.3))
                    .frame(width: 16, alignment: .center)

                Text(label)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(isHovered ? .primary : .secondary)

                Spacer(minLength: 12)

                shortcutHint(shortcut)
            }
            .padding(.vertical, 7)
            .padding(.horizontal, 10)
            .contentShape(Rectangle())
            .background(
                RoundedRectangle(cornerRadius: 7, style: .continuous)
                    .fill(.white.opacity(isHovered ? 0.06 : 0))
            )
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
            if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
        }
        .animation(.easeInOut(duration: 0.12), value: isHovered)
    }
}

