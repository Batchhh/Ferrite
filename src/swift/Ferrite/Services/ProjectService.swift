import AppKit
import Observation

struct ProjectStore: Codable {
    var projects: [Project]
    var tags: [ProjectTag]
}

/// Main-actor observable service for project persistence, tag management, and assembly lifecycle.
@MainActor
@Observable
final class ProjectService {
    var projects: [Project] = []
    var currentProject: Project?
    var showingProjectManager = false
    var showingNewProject = false
    var availableTags: [ProjectTag] = []
    var activeTagFilters: Set<UUID> = []
    var lastError: String?

    var filteredProjects: [Project] {
        if activeTagFilters.isEmpty { return projects }
        return projects.filter { project in
            !activeTagFilters.isDisjoint(with: project.tags)
        }
    }

    var storageURL: URL {
        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let dir = appSupport.appendingPathComponent("Ferrite", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("projects.json")
    }

    init() {
        load()
    }

    // MARK: - Project CRUD

    @discardableResult
    func createProject(name: String, tags: [UUID] = []) -> Project {
        var project = Project(name: name)
        project.tags = tags
        projects.insert(project, at: 0)
        save()
        return project
    }

    /// Open a project: update its `lastOpenedAt`, clear the decompiler, and reload its assemblies.
    func openProject(_ project: Project, in service: DecompilerService) {
        if let idx = projects.firstIndex(where: { $0.id == project.id }) {
            projects[idx].lastOpenedAt = Date()
            currentProject = projects[idx]
        } else {
            currentProject = project
        }
        save()
        service.clearAll()
        let urls = (currentProject?.dllPaths ?? []).map { URL(fileURLWithPath: $0) }
        service.loadAssemblies(urls: urls)
    }

    func closeProject(in service: DecompilerService) {
        currentProject = nil
        service.clearAll()
    }

    func deleteProject(_ project: Project, in service: DecompilerService) {
        if currentProject?.id == project.id {
            closeProject(in: service)
        }
        projects.removeAll { $0.id == project.id }
        save()
    }

    // MARK: - Assembly management

    /// Load an assembly into the decompiler and persist its path in the current project.
    func addAssembly(url: URL, in service: DecompilerService) {
        service.loadAssembly(url: url)
        guard let project = currentProject,
              let idx = projects.firstIndex(where: { $0.id == project.id }) else { return }
        let path = url.path
        guard !projects[idx].dllPaths.contains(path) else { return }
        projects[idx].dllPaths.append(path)
        currentProject = projects[idx]
        save()
    }

    /// Unload an assembly from the decompiler and remove its path from the current project.
    func removeAssembly(id: String, filePath: String, in service: DecompilerService) {
        service.removeAssembly(id: id)
        guard let project = currentProject,
              let idx = projects.firstIndex(where: { $0.id == project.id }) else { return }
        projects[idx].dllPaths.removeAll { $0 == filePath }
        currentProject = projects[idx]
        save()
    }

    // MARK: - Tag management

    @discardableResult
    func createTag(name: String, color: TagColor) -> ProjectTag {
        let tag = ProjectTag(name: name, color: color)
        availableTags.append(tag)
        save()
        return tag
    }

    func deleteTag(id: UUID) {
        availableTags.removeAll { $0.id == id }
        for i in projects.indices {
            projects[i].tags.removeAll { $0 == id }
        }
        if currentProject != nil, let idx = projects.firstIndex(where: { $0.id == currentProject!.id }) {
            currentProject = projects[idx]
        }
        save()
    }

    func addTag(_ tagId: UUID, to projectId: UUID) {
        guard let idx = projects.firstIndex(where: { $0.id == projectId }) else { return }
        guard !projects[idx].tags.contains(tagId) else { return }
        projects[idx].tags.append(tagId)
        if currentProject?.id == projectId { currentProject = projects[idx] }
        save()
    }

    func removeTag(_ tagId: UUID, from projectId: UUID) {
        guard let idx = projects.firstIndex(where: { $0.id == projectId }) else { return }
        projects[idx].tags.removeAll { $0 == tagId }
        if currentProject?.id == projectId { currentProject = projects[idx] }
        save()
    }

    func toggleTagFilter(_ tagId: UUID) {
        if activeTagFilters.contains(tagId) { activeTagFilters.remove(tagId) }
        else { activeTagFilters.insert(tagId) }
    }

    func clearFilters() { activeTagFilters.removeAll() }

    func tags(for project: Project) -> [ProjectTag] {
        project.tags.compactMap { tagId in
            availableTags.first { $0.id == tagId }
        }
    }

    // MARK: - Item Tags

    /// Load item tags from the current project, converting raw values to `ItemTag`.
    func loadItemTags() -> [String: Set<ItemTag>] {
        guard let project = currentProject else { return [:] }
        var result: [String: Set<ItemTag>] = [:]
        for (key, rawValues) in project.itemTags {
            let tags = Set(rawValues.compactMap { ItemTag(rawValue: $0) })
            if !tags.isEmpty { result[key] = tags }
        }
        return result
    }

    /// Write item tags back to the current project and persist.
    func updateItemTags(_ tags: [String: Set<ItemTag>]) {
        guard let project = currentProject,
              let idx = projects.firstIndex(where: { $0.id == project.id }) else { return }
        var encoded: [String: [String]] = [:]
        for (key, tagSet) in tags {
            if !tagSet.isEmpty { encoded[key] = tagSet.map(\.rawValue) }
        }
        projects[idx].itemTags = encoded
        currentProject = projects[idx]
        save()
    }
}
