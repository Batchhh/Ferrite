import SwiftUI
import AppKit
import UniformTypeIdentifiers

// Requires macOS 26 (Tahoe) for Liquid Glass APIs.
struct ContentView: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @Environment(SearchService.self) private var searchService
    @State private var showSidebar = true
    @State private var isFullScreen = false

    private var showsCode: Bool {
        switch service.selection {
        case .type, .member: return true
        default: return false
        }
    }

    private var isWelcomeScreen: Bool {
        projectService.currentProject == nil && service.loadedAssemblies.isEmpty
    }

    var body: some View {
        mainContent
            .background { keyboardShortcuts }
    }

    // Extracted to keep body type-check fast
    private var mainContent: some View {
        splitView
            .overlay { modalOverlays }
            .overlay {
                if let batchState = service.batchLoading {
                    BatchLoadingOverlay(state: batchState)
                        .transition(.asymmetric(
                            insertion: .identity,
                            removal: .opacity
                        ))
                }
            }
            .animation(.easeOut(duration: 0.3), value: service.batchLoading != nil)
            .onChange(of: service.loadedAssemblies.count) {
                guard service.batchLoading == nil else { return }
                searchService.rebuildIndex(from: service.loadedAssemblies, session: service.session)
            }
            .onChange(of: service.batchLoading) { oldValue, newValue in
                if oldValue != nil && newValue == nil {
                    searchService.rebuildIndex(from: service.loadedAssemblies, session: service.session)
                }
            }
            .onChange(of: searchService.isPresented) { _, isPresented in
                guard isPresented else { return }
                if projectService.showingProjectManager {
                    projectService.showingProjectManager = false
                }
                if projectService.showingNewProject {
                    projectService.showingNewProject = false
                }
            }
            .onChange(of: projectService.showingProjectManager) { _, isPresented in
                guard isPresented else { return }
                dismissSearchIfNeeded()
                if projectService.showingNewProject {
                    projectService.showingNewProject = false
                }
            }
            .onChange(of: projectService.showingNewProject) { _, isPresented in
                guard isPresented else { return }
                dismissSearchIfNeeded()
            }
            .onDrop(of: [.fileURL], isTargeted: nil) { providers in
                handleDrop(providers)
            }
    }

    @ViewBuilder
    private var modalOverlays: some View {
        let projectOverlayPresented = projectService.showingProjectManager || projectService.showingNewProject

        if searchService.isPresented && !projectOverlayPresented {
            SearchPanel()
        }
        if projectService.showingProjectManager {
            ProjectManagerOverlay()
        }
        if projectService.showingNewProject && !projectService.showingProjectManager {
            NewProjectOverlay()
        }
    }

    private var keyboardShortcuts: some View {
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

    static let titleBarHeight: CGFloat = 44

    // Extracted to keep body type-check fast
    private var splitView: some View {
        HStack(spacing: 0) {
            if showSidebar && !isWelcomeScreen && service.batchLoading == nil {
                // Sidebar column — full height including title bar area
                VStack(spacing: 0) {
                    sidebarTitleBar
                    UpdateBannerView()
                    AssemblyTreeView()
                        .frame(maxHeight: .infinity)
                }
                .frame(minWidth: 200, idealWidth: 260, maxWidth: 400)
                .background(.ultraThinMaterial)
            }

            // Detail column — full height including title bar area
            VStack(spacing: 0) {
                detailTitleBar
                detailContent
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            .background(isWelcomeScreen ? Color(red: 25/255, green: 25/255, blue: 28/255) : Color.contentBackground)
            .clipShape(Rectangle())
        }
        .ignoresSafeArea(edges: .top)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(.ultraThinMaterial)
        .background(WindowConfigurator(isFullScreen: $isFullScreen))
    }

    private var trafficLightSpacer: some View {
        Color.clear
            .frame(width: isFullScreen ? 12 : 90)
            .animation(.easeInOut(duration: 0.3), value: isFullScreen)
    }

    /// Title bar area above the sidebar — traffic lights + sidebar toggle.
    private var sidebarTitleBar: some View {
        HStack(spacing: 0) {
            trafficLightSpacer

            TitleBarButton(icon: "sidebar.left") {
                withAnimation(.spring(duration: 0.3, bounce: 0.1)) { showSidebar.toggle() }
            }

            Spacer(minLength: 0)
        }
        .padding(.top, 4)
        .frame(height: Self.titleBarHeight)
    }

    /// Title bar area above the detail pane — draggable + breadcrumb + export button.
    private var detailTitleBar: some View {
        HStack(spacing: 0) {
            if !showSidebar {
                trafficLightSpacer

                TitleBarButton(icon: "sidebar.left") {
                    withAnimation(.spring(duration: 0.3, bounce: 0.1)) { showSidebar.toggle() }
                }

                if showsCode {
                    RoundedRectangle(cornerRadius: 0.5)
                        .fill(.white.opacity(0.1))
                        .frame(width: 1, height: 16)
                        .padding(.leading, 14)
                        .padding(.trailing, 2)
                }
            }

            if showsCode {
                BreadcrumbBar()
            } else {
                WindowDragArea()
            }

        }
        .padding(.top, 4)
        .frame(height: Self.titleBarHeight)
    }

    @ViewBuilder
    private var detailContent: some View {
        if service.batchLoading != nil {
            Color(red: 25/255, green: 25/255, blue: 28/255)
        } else if projectService.currentProject == nil && service.loadedAssemblies.isEmpty {
            WelcomeView()
        } else if service.loadedAssemblies.isEmpty {
            EmptyProjectView()
        } else if case .assembly = service.selection {
            DetailView()
        } else if case .namespace = service.selection {
            DetailView()
        } else {
            CodePreviewView()
        }
    }

    private func handleDrop(_ providers: [NSItemProvider]) -> Bool {
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

    private func dismissSearchIfNeeded() {
        guard searchService.isPresented else { return }
        searchService.isPresented = false
        searchService.query = ""
        searchService.performSearch()
    }
}
