import SwiftUI

@main
struct FerriteApp: App {
    @State private var service = DecompilerService()
    @State private var projectService = ProjectService()
    @State private var searchService = SearchService()
    @State private var itemTagService = ItemTagService()
    @State private var updateService = UpdateService()

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
            CommandGroup(replacing: .newItem) {
                Button("Open Assembly...") {
                    projectService.showOpenPanel(in: service)
                }
                .keyboardShortcut("o", modifiers: .command)
            }
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
