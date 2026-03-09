import SwiftUI
import AppKit
import UniformTypeIdentifiers

extension ContentView {
    var keyboardShortcuts: some View {
        Group {
            // Cmd+P opens project manager (distinct from Cmd+Shift+P in menu)
            Button("") { projectService.showingProjectManager.toggle() }
                .keyboardShortcut("p", modifiers: .command)
        }
        .hidden()
    }

    func handleDrop(_ providers: [NSItemProvider]) -> Bool {
        let service = service
        let projectService = projectService
        Task { @MainActor in
            var urls: [URL] = []
            for provider in providers {
                if let item = try? await provider.loadItem(forTypeIdentifier: "public.file-url", options: nil),
                   let data = item as? Data,
                   let url = URL(dataRepresentation: data, relativeTo: nil) {
                    let ext = url.pathExtension.lowercased()
                    if ext == "dll" || ext == "exe" {
                        urls.append(url)
                    }
                }
            }
            if urls.count >= 20 {
                service.loadAssemblies(urls: urls)
                if let project = projectService.currentProject,
                   let idx = projectService.projects.firstIndex(where: { $0.id == project.id }) {
                    for url in urls {
                        let path = url.path
                        if !projectService.projects[idx].dllPaths.contains(path) {
                            projectService.projects[idx].dllPaths.append(path)
                        }
                    }
                    projectService.currentProject = projectService.projects[idx]
                }
            } else {
                for url in urls {
                    projectService.addAssembly(url: url, in: service)
                }
            }
        }
        return !providers.isEmpty
    }

    func dismissSearchIfNeeded() {
        guard searchService.isPresented else { return }
        searchService.isPresented = false
        searchService.query = ""
        searchService.performSearch()
    }
}
