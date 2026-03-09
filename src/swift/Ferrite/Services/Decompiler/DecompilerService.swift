import SwiftUI
import UniformTypeIdentifiers

enum CodeLanguage: String, CaseIterable {
    case csharp = "C#"
    case il = "IL"
}

/// Main-actor observable service for loaded assemblies, selection state, and decompilation.
@MainActor
@Observable
final class DecompilerService {
    struct BatchLoadingState: Equatable {
        var totalCount: Int
        var completedCount: Int
        var currentAssemblyName: String
        var progress: Double { totalCount > 0 ? Double(completedCount) / Double(totalCount) : 0 }
    }

    var loadedAssemblies: [AssemblySummary] = []
    var selection: Selection?
    var codeLanguage: CodeLanguage = .csharp
    var isLoading = false
    var error: String?
    var batchLoading: BatchLoadingState? = nil

    /// Tracks which tree nodes are expanded, keyed by a stable identifier string.
    var expandedNodes: Set<String> = []

    /// Whether all code blocks in the code preview are currently expanded.
    var codeAllExpanded = false
    /// Incremented to trigger a toggle in CodePreviewView from the title bar button.
    var codeCollapseToggleId = 0
    /// Incremented to trigger sidebar toggle from the menu bar.
    var sidebarToggleId = 0
    /// Incremented to trigger code search toggle from the menu bar.
    var codeSearchToggleId = 0

    /// Short type name -> list of (assemblyId, token) for navigation lookups.
    var typeNameCache: [String: [(assemblyId: String, token: UInt32)]] = [:]
    var namespaceTypesCache: [String: [TypeSummary]] = [:]
    var typeDetailsCache: [String: TypeInfo] = [:]

    func isExpanded(_ nodeId: String) -> Bool { expandedNodes.contains(nodeId) }

    func toggleExpanded(_ nodeId: String) {
        if expandedNodes.contains(nodeId) { expandedNodes.remove(nodeId) }
        else { expandedNodes.insert(nodeId) }
    }

    let session = DecompilerSession()

    /// Unload all assemblies and reset selection and tree state.
    func clearAll() {
        for assembly in loadedAssemblies {
            do {
                try session.removeAssembly(id: assembly.id)
            } catch {
                self.error = error.localizedDescription
            }
        }
        loadedAssemblies = []
        expandedNodes = []
        selection = nil
        typeNameCache = [:]
        namespaceTypesCache = [:]
        typeDetailsCache = [:]
    }

    /// Load a .NET assembly from `url` on a background task, then update state on the main actor.
    func loadAssembly(url: URL) {
        let path = url.path

        // Prevent loading the same file twice.
        if let existing = loadedAssemblies.first(where: { $0.filePath == path }) {
            selection = .assembly(id: existing.id)
            return
        }

        isLoading = true
        error = nil
        let session = self.session
        Task.detached { [weak self] in
            do {
                let summary = try session.loadAssemblyLazy(path: path)
                let summaryId = summary.id
                await MainActor.run { [weak self] in
                    guard let self else { return }
                    self.loadedAssemblies.append(summary)
                    self.addToTypeNameCache(summary)
                    self.selection = .assembly(id: summaryId)
                    self.isLoading = false
                }
            } catch {
                let message = error.localizedDescription
                await MainActor.run { [weak self] in
                    guard let self else { return }
                    self.error = message
                    self.isLoading = false
                }
            }
        }
    }

    /// Load multiple assemblies concurrently with batch progress tracking.
    func loadAssemblies(urls: [URL]) {
        let paths = urls.map { $0.path }
        let newPaths = paths.filter { path in
            !loadedAssemblies.contains { $0.filePath == path }
        }
        guard !newPaths.isEmpty else { return }

        batchLoading = BatchLoadingState(
            totalCount: newPaths.count,
            completedCount: 0,
            currentAssemblyName: URL(fileURLWithPath: newPaths[0]).lastPathComponent
        )
        error = nil
        let session = self.session

        Task.detached { [weak self] in
            let results = await withTaskGroup(
                of: (Int, Result<AssemblySummary, Error>).self,
                returning: [(Int, Result<AssemblySummary, Error>)].self
            ) { group in
                for (index, path) in newPaths.enumerated() {
                    group.addTask {
                        do {
                            let summary = try session.loadAssemblyLazy(path: path)
                            return (index, .success(summary))
                        } catch {
                            return (index, .failure(error))
                        }
                    }
                }
                var collected: [(Int, Result<AssemblySummary, Error>)] = []
                var completedCount = 0
                for await result in group {
                    completedCount += 1
                    let count = completedCount
                    collected.append(result)
                    await MainActor.run { [weak self] in
                        self?.batchLoading?.completedCount = count
                    }
                }
                return collected
            }

            var summaries: [AssemblySummary] = []
            var cacheEntries: [(String, (assemblyId: String, token: UInt32))] = []
            var firstError: String?

            for (_, result) in results {
                switch result {
                case .success(let summary):
                    do {
                        let items = try session.getSearchableItems(assemblyId: summary.id)
                        for item in items where item.parentToken == nil {
                            let entry = (assemblyId: summary.id, token: item.token)
                            cacheEntries.append((item.name, entry))
                            cacheEntries.append((item.fullName, entry))
                            if let backtickIdx = item.name.firstIndex(of: "`") {
                                let cleanName = String(item.name[..<backtickIdx])
                                cacheEntries.append((cleanName, entry))
                            }
                        }
                    } catch {
                        if firstError == nil { firstError = error.localizedDescription }
                    }
                    summaries.append(summary)
                case .failure(let error):
                    if firstError == nil { firstError = error.localizedDescription }
                }
            }
            await MainActor.run { [weak self] in
                guard let self else { return }
                self.loadedAssemblies.append(contentsOf: summaries)
                for (key, entry) in cacheEntries {
                    self.typeNameCache[key, default: []].append(entry)
                }
                if let firstError { self.error = firstError }
                self.batchLoading = nil
                if self.selection == nil, let first = self.loadedAssemblies.first {
                    self.selection = .assembly(id: first.id)
                }
            }
        }
    }

    /// Unload a single assembly by ID, pruning its expanded nodes and clearing selection if needed.
    func removeAssembly(id: String) {
        do {
            try session.removeAssembly(id: id)
        } catch {
            self.error = error.localizedDescription
        }
        loadedAssemblies.removeAll { $0.id == id }
        expandedNodes = expandedNodes.filter { !$0.contains(id) }
        if case .assembly(let selectedId) = selection, selectedId == id {
            selection = nil
        }
        // Clear caches for this assembly
        namespaceTypesCache = namespaceTypesCache.filter { !$0.key.hasPrefix("\(id):") }
        typeDetailsCache = typeDetailsCache.filter { !$0.key.hasPrefix("\(id):") }
        rebuildTypeNameCache()
    }
}
