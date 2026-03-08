import SwiftUI
import UniformTypeIdentifiers

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
    var isLoading = false
    var error: String?
    var batchLoading: BatchLoadingState? = nil

    /// Tracks which tree nodes are expanded, keyed by a stable identifier string.
    var expandedNodes: Set<String> = []

    /// Whether all code blocks in the code preview are currently expanded.
    var codeAllExpanded = false
    /// Incremented to trigger a toggle in CodePreviewView from the title bar button.
    var codeCollapseToggleId = 0

    /// Short type name → list of (assemblyId, token) for navigation lookups.
    /// Multiple assemblies may define the same short name; resolution prefers the current assembly.
    private var typeNameCache: [String: [(assemblyId: String, token: UInt32)]] = [:]

    // Lazy loading caches
    private var namespaceTypesCache: [String: [TypeSummary]] = [:]
    private var typeDetailsCache: [String: TypeInfo] = [:]

    func isExpanded(_ nodeId: String) -> Bool {
        expandedNodes.contains(nodeId)
    }

    func toggleExpanded(_ nodeId: String) {
        if expandedNodes.contains(nodeId) {
            expandedNodes.remove(nodeId)
        } else {
            expandedNodes.insert(nodeId)
        }
    }

    let session = DecompilerSession()

    /// Unload all assemblies and reset selection and tree state.
    func clearAll() {
        for assembly in loadedAssemblies {
            try? session.removeAssembly(id: assembly.id)
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
                    if let items = try? session.getSearchableItems(assemblyId: summary.id) {
                        for item in items where item.parentToken == nil {
                            let entry = (assemblyId: summary.id, token: item.token)
                            cacheEntries.append((item.name, entry))
                            cacheEntries.append((item.fullName, entry))
                            if let backtickIdx = item.name.firstIndex(of: "`") {
                                let cleanName = String(item.name[..<backtickIdx])
                                cacheEntries.append((cleanName, entry))
                            }
                        }
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
        try? session.removeAssembly(id: id)
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

    // MARK: - Lazy loading accessors

    /// Fetch type summaries for a namespace (cached after first call).
    func getNamespaceTypes(assemblyId: String, namespace: String) -> [TypeSummary] {
        let key = "\(assemblyId):\(namespace)"
        if let cached = namespaceTypesCache[key] { return cached }
        do {
            let types = try session.getNamespaceTypes(assemblyId: assemblyId, namespace: namespace)
            namespaceTypesCache[key] = types
            return types
        } catch {
            print("Failed to fetch namespace types: \(error)")
            return []
        }
    }

    /// Fetch full type details by token (cached after first call).
    func getTypeDetails(assemblyId: String, token: UInt32) -> TypeInfo? {
        let key = "\(assemblyId):\(token)"
        if let cached = typeDetailsCache[key] { return cached }
        do {
            let info = try session.getTypeDetails(assemblyId: assemblyId, typeToken: token)
            typeDetailsCache[key] = info
            return info
        } catch {
            print("Failed to fetch type details: \(error)")
            return nil
        }
    }

    // MARK: - Lookup helpers

    func findAssembly(id: String) -> AssemblySummary? {
        loadedAssemblies.first { $0.id == id }
    }

    func findType(assemblyId: String, token: UInt32) -> TypeInfo? {
        getTypeDetails(assemblyId: assemblyId, token: token)
    }

    func findMember(assemblyId: String, typeToken: UInt32, memberToken: UInt32) -> MemberInfo? {
        guard let type_ = findType(assemblyId: assemblyId, token: typeToken) else { return nil }
        return type_.members.first { $0.token == memberToken }
    }

    func findTypeByShortName(_ name: String, preferredAssemblyId: String? = nil) -> (assemblyId: String, token: UInt32)? {
        guard let entries = typeNameCache[name], !entries.isEmpty else { return nil }
        if let preferred = preferredAssemblyId,
           let match = entries.first(where: { $0.assemblyId == preferred }) {
            return match
        }
        return entries.first
    }

    // MARK: - Type Name Cache

    /// Rebuild the type name cache from scratch after an assembly is removed.
    private func rebuildTypeNameCache() {
        typeNameCache = [:]
        for entry in loadedAssemblies {
            addToTypeNameCache(entry)
        }
    }

    func addToTypeNameCache(_ entry: AssemblySummary) {
        do {
            let items = try session.getSearchableItems(assemblyId: entry.id)
            for item in items where item.parentToken == nil {
                let cacheEntry = (assemblyId: entry.id, token: item.token)
                typeNameCache[item.name, default: []].append(cacheEntry)
                typeNameCache[item.fullName, default: []].append(cacheEntry)
                if let backtickIdx = item.name.firstIndex(of: "`") {
                    let cleanName = String(item.name[..<backtickIdx])
                    typeNameCache[cleanName, default: []].append(cacheEntry)
                }
            }
        } catch {
            print("Failed to fetch searchable items: \(error)")
        }
    }

    /// Decompile a type to C# source, returning `nil` on failure.
    func decompileType(assemblyId: String, token: UInt32) -> String? {
        do {
            return try session.decompileType(assemblyId: assemblyId, typeToken: token)
        } catch {
            print("Decompilation failed: \(error)")
            return nil
        }
    }
}
