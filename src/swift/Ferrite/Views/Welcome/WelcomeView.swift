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
        VStack(alignment: .leading, spacing: 32) {
            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text("Ferrite")
                        .font(.system(size: 26, weight: .semibold))
                        .foregroundStyle(.primary)

                    Text("v\(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.1.0")")
                        .font(.system(size: 11, weight: .medium, design: .monospaced))
                        .foregroundStyle(.white.opacity(0.25))
                }

                Text(".NET Disassembler")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundStyle(.white.opacity(0.3))
                    .tracking(1.2)
                    .textCase(.uppercase)
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
                    HStack {
                        Text("Recent")
                            .font(.system(size: 11, weight: .medium))
                            .foregroundStyle(.white.opacity(0.25))
                            .tracking(0.6)
                            .textCase(.uppercase)
                        Spacer()
                    }
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

// MARK: - Welcome Project Row

struct WelcomeProjectRow: View {
    let project: Project
    let onOpen: () -> Void
    let onDelete: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: onOpen) {
            HStack(spacing: 8) {
                Image(systemName: "doc.text")
                    .font(.system(size: 10))
                    .foregroundColor(isHovered ? .secondary : Color.white.opacity(0.2))
                    .frame(width: 16, alignment: .center)

                Text(project.name)
                    .font(.system(size: 13))
                    .foregroundStyle(isHovered ? .primary : .tertiary)
                    .lineLimit(1)
                    .truncationMode(.tail)

                Spacer(minLength: 12)

                Text(relativeDateText)
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(.white.opacity(0.15))
            }
            .padding(.vertical, 6)
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
        .contextMenu {
            Button("Delete Project", role: .destructive, action: onDelete)
        }
    }

    private var relativeDateText: String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: project.lastOpenedAt, relativeTo: Date())
    }
}

// MARK: - Drop Zone Card

struct DropZoneCard: View {
    @State private var iconOffset: CGFloat = 0

    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "arrow.down.circle.fill")
                .font(.system(size: 30, weight: .regular))
                .foregroundStyle(Color(NSColor.controlAccentColor))
                .offset(y: iconOffset)
                .animation(
                    .easeInOut(duration: 1.2).repeatForever(autoreverses: true),
                    value: iconOffset
                )

            Text("Drop DLL or EXE to Open")
                .font(.system(size: 17, weight: .semibold))
                .foregroundStyle(.primary)

            Text("Ferrite will load the assembly immediately.")
                .font(.system(size: 13))
                .foregroundStyle(.secondary)
        }
        .multilineTextAlignment(.center)
        .padding(.horizontal, 30)
        .padding(.vertical, 24)
        .background(
            RoundedRectangle(cornerRadius: 26, style: .continuous)
                .fill(.black.opacity(0.42))
        )
        .overlay {
            RoundedRectangle(cornerRadius: 26, style: .continuous)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        }
        .shadow(color: .black.opacity(0.25), radius: 24, y: 10)
        .onAppear { iconOffset = -4 }
    }
}

// MARK: - Keyboard Shortcut Hint

func shortcutHint(_ text: String) -> some View {
    Text(text)
        .font(.system(size: 11, weight: .medium, design: .rounded))
        .foregroundStyle(.quaternary)
        .frame(height: 22)
        .padding(.horizontal, 6)
        .background(
            RoundedRectangle(cornerRadius: 5)
                .fill(.white.opacity(0.06))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 5)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        )
}
