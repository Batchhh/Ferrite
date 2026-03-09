import SwiftUI
import AppKit
import UniformTypeIdentifiers

// Requires macOS 26 (Tahoe) for Liquid Glass APIs.
struct ContentView: View {
    @Environment(DecompilerService.self) var service
    @Environment(ProjectService.self) var projectService
    @Environment(SearchService.self) var searchService
    @State var showSidebar = true
    @State var isFullScreen = false

    var showsCode: Bool {
        switch service.selection {
        case .type, .member: return true
        default: return false
        }
    }

    var isWelcomeScreen: Bool {
        projectService.currentProject == nil && service.loadedAssemblies.isEmpty
    }

    var body: some View {
        mainContent
            .background { keyboardShortcuts }
            .onChange(of: service.sidebarToggleId) {
                withAnimation(.spring(duration: 0.3, bounce: 0.1)) {
                    showSidebar.toggle()
                }
            }
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

                languageToggle
                    .padding(.trailing, 14)
            } else {
                WindowDragArea()
            }

        }
        .padding(.top, 4)
        .frame(height: Self.titleBarHeight)
    }

    private var languageToggle: some View {
        LanguageToggleButton(language: service.codeLanguage) {
            withAnimation(.easeInOut(duration: 0.2)) {
                service.codeLanguage = service.codeLanguage == .csharp ? .il : .csharp
            }
        }
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

}
