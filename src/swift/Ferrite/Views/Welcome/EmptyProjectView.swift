import SwiftUI
import UniformTypeIdentifiers

struct EmptyProjectView: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @State private var isTargeted = false

    var body: some View {
        VStack(spacing: 0) {
            Spacer()
            centerSection
            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .overlay {
            if isTargeted {
                DropZoneCard()
                    .transition(.opacity.combined(with: .scale(scale: 0.92)))
            }
        }
        .animation(.easeInOut(duration: 0.15), value: isTargeted)
        .onDrop(of: [.fileURL], isTargeted: $isTargeted) { providers in
            handleDrop(providers)
        }
    }

    private var centerSection: some View {
        VStack(spacing: 12) {
            Text("Drop .dll or .exe files here")
                .font(.system(size: 13))
                .foregroundStyle(.secondary)

            HStack(spacing: 6) {
                Text("or press")
                    .font(.system(size: 13))
                    .foregroundStyle(.quaternary)

                KeyCapView("⌘O")

                Text("to add assemblies")
                    .font(.system(size: 13))
                    .foregroundStyle(.quaternary)
            }
        }
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
