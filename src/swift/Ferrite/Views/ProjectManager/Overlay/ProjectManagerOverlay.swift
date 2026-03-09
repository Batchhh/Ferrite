import SwiftUI

// MARK: - Project Manager Overlay

struct ProjectManagerOverlay: View {
    @Environment(DecompilerService.self) var service
    @Environment(ProjectService.self) var projectService
    @State var isVisible = false
    @State var showFilters = false
    @State var query = ""
    @State var selectedIndex: Int = 0
    @FocusState var isFieldFocused: Bool

    var hasActiveFilters: Bool { !projectService.activeTagFilters.isEmpty }

    var filteredProjects: [Project] {
        let projects = projectService.filteredProjects
        if query.isEmpty { return projects }
        return projects.filter { $0.name.localizedCaseInsensitiveContains(query) }
    }

    var body: some View {
        ZStack {
            Color.black.opacity(isVisible ? 0.25 : 0)
                .ignoresSafeArea()
                .onTapGesture {
                    if showFilters {
                        withAnimation(.spring(duration: 0.25, bounce: 0.1)) {
                            showFilters = false
                        }
                    } else if projectService.showingNewProject {
                        withAnimation(.spring(duration: 0.25, bounce: 0.1)) {
                            projectService.showingNewProject = false
                        }
                    } else {
                        dismiss()
                    }
                }
                .animation(.easeOut(duration: 0.2), value: isVisible)

            projectManagerContent
                .overlay(alignment: .topTrailing) {
                    if showFilters && !projectService.availableTags.isEmpty {
                        filterPopover
                            .offset(x: -18, y: 52)
                            .transition(.opacity.combined(with: .scale(scale: 0.95, anchor: .topTrailing)))
                    }
                }
                .scaleEffect(isVisible ? 1 : 0.95)
                .opacity(isVisible ? 1 : 0)
                .padding(.top, OverlayLayout.topPadding)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
                .animation(.spring(duration: 0.25, bounce: 0.15), value: isVisible)

            if projectService.showingNewProject {
                newProjectContent
                    .transition(.opacity.combined(with: .scale(scale: 0.95)))
                    .zIndex(1)
            }
        }
        .onAppear {
            selectedIndex = 0
            isVisible = true
            focusSearchField()
        }
        .onChange(of: projectService.showingNewProject) { _, isPresented in
            if isPresented {
                showFilters = false
            } else {
                focusSearchField()
            }
        }
    }

    // MARK: - Container

    private var projectManagerContent: some View {
        VStack(spacing: 0) {
            searchBar

            if hasActiveFilters {
                activeFilterChips
            }

            if !filteredProjects.isEmpty || !query.isEmpty {
                Divider().opacity(0.5)
                projectList
            } else if projectService.projects.isEmpty {
                Divider().opacity(0.5)
                emptyState
            }
        }
        .frame(width: 520)
        .fixedSize(horizontal: false, vertical: true)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 14))
        .overlay(
            RoundedRectangle(cornerRadius: 14)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        )
        .shadow(color: .black.opacity(0.45), radius: 40, y: 12)
    }

    // MARK: - Search Bar

    private var searchBar: some View {
        SearchBarView(
            query: $query,
            hasActiveFilters: hasActiveFilters,
            showFilters: showFilters,
            activeFilterCount: projectService.activeTagFilters.count,
            hasTags: !projectService.availableTags.isEmpty,
            onReturn: openSelected,
            onMoveUp: {
                withAnimation(.easeOut(duration: 0.1)) {
                    if selectedIndex > 0 { selectedIndex -= 1 }
                }
            },
            onMoveDown: {
                withAnimation(.easeOut(duration: 0.1)) {
                    if selectedIndex < filteredProjects.count - 1 { selectedIndex += 1 }
                }
            },
            onDeleteSelected: deleteSelected,
            onEscape: {
                if showFilters {
                    withAnimation(.spring(duration: 0.25, bounce: 0.1)) { showFilters = false }
                } else if projectService.showingNewProject {
                    withAnimation(.spring(duration: 0.25, bounce: 0.1)) { projectService.showingNewProject = false }
                } else {
                    dismiss()
                }
            },
            onToggleFilters: {
                withAnimation(.spring(duration: 0.25, bounce: 0.1)) { showFilters.toggle() }
            },
            onNewProject: {
                withAnimation(.spring(duration: 0.25, bounce: 0.1)) { projectService.showingNewProject = true }
            },
            isFieldFocused: $isFieldFocused
        )
        .onChange(of: query) { selectedIndex = 0 }
    }

}
