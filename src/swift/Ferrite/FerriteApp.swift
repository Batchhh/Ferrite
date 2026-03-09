import SwiftUI

@main
struct FerriteApp: App {
    @State private var service = DecompilerService()
    @State private var projectService = ProjectService()
    @State private var searchService = SearchService()
    @State private var itemTagService = ItemTagService()
    @State private var updateService = UpdateService()

    private var hasCodeSelection: Bool {
        switch service.selection {
        case .type, .member: return true
        default: return false
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(service)
                .environment(projectService)
                .environment(searchService)
                .environment(itemTagService)
                .environment(updateService)
                .onChange(of: projectService.currentProject?.id, initial: true) {
                    itemTagService.load(from: projectService)
                }
                .onAppear {
                    if let window = NSApplication.shared.windows.first {
                        window.zoom(nil)
                    }
                }
                .task {
                    await updateService.checkForUpdates()
                }
        }
        .windowStyle(.hiddenTitleBar)
        .commands {
            // MARK: - File

            CommandGroup(replacing: .newItem) {
                Button("Open Assembly...") {
                    projectService.showOpenPanel(in: service)
                }
                .keyboardShortcut("o", modifiers: .command)

                Divider()

                Button("Export Code...") {
                    projectService.exportCode(selection: service.selection, in: service)
                }
                .keyboardShortcut("e", modifiers: .command)
                .disabled(!hasCodeSelection)

                Button("Export Header...") {
                    projectService.exportHeader(selection: service.selection, in: service)
                }
                .keyboardShortcut("e", modifiers: [.command, .shift])
                .disabled(!hasCodeSelection)
            }

            // Remove system "Show Tab Bar" / "Show All Tabs"
            CommandGroup(replacing: .toolbar) {}
            CommandGroup(replacing: .windowArrangement) {}

            // MARK: - View (merged into system View menu)

            CommandGroup(before: .windowSize) {
                Button("Toggle Sidebar") {
                    service.sidebarToggleId += 1
                }
                .keyboardShortcut("b", modifiers: .command)

                Button("Search Assemblies") {
                    searchService.isPresented.toggle()
                }
                .keyboardShortcut("k", modifiers: .command)

                Divider()

                Button("Switch Language") {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        service.codeLanguage = service.codeLanguage == .csharp ? .il : .csharp
                    }
                }
                .keyboardShortcut("l", modifiers: .command)
                .disabled(!hasCodeSelection)

                Divider()

                Button("Find in Code...") {
                    service.codeSearchToggleId += 1
                }
                .keyboardShortcut("f", modifiers: .command)
                .disabled(!hasCodeSelection)

                Button("Expand / Collapse All") {
                    service.codeCollapseToggleId += 1
                }
                .keyboardShortcut("r", modifiers: .command)
                .disabled(!hasCodeSelection)

                Divider()
            }

            // MARK: - Project

            CommandMenu("Project") {
                Button("Project Manager...") {
                    projectService.showingProjectManager = true
                }
                .keyboardShortcut("p", modifiers: [.command, .shift])

                Divider()

                Button("New Project...") {
                    projectService.showingNewProject = true
                }
                .keyboardShortcut("n", modifiers: .command)

                Divider()

                Button("Close Project") {
                    projectService.closeProject(in: service)
                }
                .disabled(projectService.currentProject == nil)
            }
        }
    }
}
