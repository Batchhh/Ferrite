import SwiftUI

extension ProjectManagerOverlay {
    // MARK: - Filter Popover

    var filterPopover: some View {
        FilterPopoverView(
            availableTags: projectService.availableTags,
            activeTagFilters: projectService.activeTagFilters,
            onToggleTag: { id in
                withAnimation(.easeInOut(duration: 0.15)) {
                    projectService.toggleTagFilter(id)
                    selectedIndex = 0
                }
            },
            onClearFilters: {
                withAnimation(.easeInOut(duration: 0.15)) {
                    projectService.clearFilters()
                    selectedIndex = 0
                }
            }
        )
    }

    // MARK: - Active Filter Chips

    var activeFilterChips: some View {
        ActiveFilterChipsView(
            availableTags: projectService.availableTags,
            activeTagFilters: projectService.activeTagFilters,
            onToggleTag: { id in
                withAnimation(.easeOut(duration: 0.15)) {
                    projectService.toggleTagFilter(id)
                    selectedIndex = 0
                }
            },
            onClearFilters: {
                withAnimation(.easeOut(duration: 0.15)) {
                    projectService.clearFilters()
                    selectedIndex = 0
                }
            }
        )
    }

    // MARK: - Project List

    var projectList: some View {
        ProjectListView(
            filteredProjects: filteredProjects,
            query: query,
            selectedIndex: selectedIndex,
            currentProjectId: projectService.currentProject?.id,
            onDelete: { project in
                withAnimation(.easeOut(duration: 0.15)) {
                    projectService.deleteProject(project, in: service)
                }
                if selectedIndex >= filteredProjects.count {
                    selectedIndex = max(0, filteredProjects.count - 1)
                }
            },
            onOpen: { project in
                projectService.openProject(project, in: service)
                dismiss()
            }
        )
    }

    // MARK: - Empty State

    var emptyState: some View {
        VStack(spacing: 10) {
            Image(systemName: "folder")
                .font(.system(size: 28, weight: .ultraLight))
                .foregroundStyle(.tertiary)
            VStack(spacing: 3) {
                Text("No projects yet")
                    .font(.callout)
                    .foregroundStyle(.tertiary)
                Text("Press + to create one")
                    .font(.caption)
                    .foregroundStyle(.quaternary)
            }
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 32)
    }

    // MARK: - New Project Content

    var newProjectContent: some View {
        NewProjectSheet(onDismiss: {
            withAnimation(.spring(duration: 0.25, bounce: 0.1)) {
                projectService.showingNewProject = false
            }
        }) { name, tags in
            let project = projectService.createProject(name: name, tags: tags)
            projectService.openProject(project, in: service)
            dismiss()
        }
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 14))
        .overlay(
            RoundedRectangle(cornerRadius: 14)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        )
        .shadow(color: .black.opacity(0.45), radius: 40, y: 12)
        .padding(.top, OverlayLayout.topPadding)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
    }

    // MARK: - Actions

    func openSelected() {
        guard !filteredProjects.isEmpty,
              selectedIndex < filteredProjects.count else { return }
        let project = filteredProjects[selectedIndex]
        projectService.openProject(project, in: service)
        dismiss()
    }

    func deleteSelected() {
        guard !filteredProjects.isEmpty,
              selectedIndex < filteredProjects.count else { return }
        let project = filteredProjects[selectedIndex]
        withAnimation(.easeOut(duration: 0.15)) {
            projectService.deleteProject(project, in: service)
        }
        if selectedIndex >= filteredProjects.count {
            selectedIndex = max(0, filteredProjects.count - 1)
        }
    }

    func dismiss() {
        withAnimation(.easeIn(duration: 0.15)) {
            isVisible = false
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
            projectService.showingProjectManager = false
            projectService.showingNewProject = false
        }
    }

    func focusSearchField() {
        DispatchQueue.main.async {
            isFieldFocused = true
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            isFieldFocused = true
        }
    }
}
