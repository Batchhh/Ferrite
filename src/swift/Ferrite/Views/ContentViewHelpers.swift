import SwiftUI
import AppKit
import UniformTypeIdentifiers

extension ContentView {
    var keyboardShortcuts: some View {
        Group {
            Button("") { projectService.showOpenPanel(in: service) }
                .keyboardShortcut("o", modifiers: .command)
            Button("") { projectService.showingProjectManager.toggle() }
                .keyboardShortcut("p", modifiers: .command)
            Button("") { projectService.showingNewProject.toggle() }
                .keyboardShortcut("n", modifiers: .command)
            Button("") {
                withAnimation(.spring(duration: 0.3, bounce: 0.1)) {
                    showSidebar.toggle()
                }
            }
            .keyboardShortcut("b", modifiers: .command)
            Button("") { if showsCode { projectService.exportCode(selection: service.selection, in: service) } }
                .keyboardShortcut("e", modifiers: .command)
            Button("") { if showsCode { projectService.exportHeader(selection: service.selection, in: service) } }
                .keyboardShortcut("e", modifiers: [.command, .shift])
            Button("") { searchService.isPresented.toggle() }
                .keyboardShortcut("k", modifiers: .command)
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
