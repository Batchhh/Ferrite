import Dispatch
import SwiftUI

/// Main-actor observable service managing search state, query debouncing, and index rebuilding.
/// For small-to-medium indexes (<=30k items), scoring runs synchronously on the main thread
/// so results appear in the same frame as the keystroke. Large indexes fall back to a
/// dedicated high-priority `DispatchQueue` with 30ms debouncing.
@MainActor
@Observable
final class SearchService {
    var isPresented = false
    var query = ""
    var activeFilters: Set<SearchFilter> = []
    private(set) var results: [SearchItem] = []
    private(set) var recentItems: [SearchItem] = []

    private static let maxRecents = 10

    /// Current search index snapshot — replaced atomically when rebuilt.
    private var searchIndex: SearchIndex = .empty

    /// Dedicated high-priority queue for search scoring — used only for large indexes.
    let searchQueue = DispatchQueue(label: "ferrite.search", qos: .userInteractive)

    /// Current in-flight search work item — cancelled when a new search starts.
    var currentWork: DispatchWorkItem?

    /// Debounce work item for the async path.
    private var debounceWork: DispatchWorkItem?

    /// Indexes at or below this size are scored synchronously on the main thread.
    static let syncThreshold = 30_000

    // MARK: - Index Building

    func rebuildIndex(from assemblies: [AssemblySummary], session: DecompilerSession) {
        Task.detached(priority: .userInitiated) { [weak self] in
            let newIndex = SearchIndex.build(from: assemblies, session: session)
            await MainActor.run { [weak self] in
                guard let self else { return }
                self.searchIndex = newIndex
                if !self.query.isEmpty {
                    self.performSearch()
                }
            }
        }
    }

    // MARK: - Search

    func debouncedSearch() {
        if searchIndex.items.count <= Self.syncThreshold {
            performSearch()
        } else {
            debounceWork?.cancel()
            let work = DispatchWorkItem { [weak self] in
                self?.performSearch()
            }
            debounceWork = work
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.03, execute: work)
        }
    }

    func addRecent(_ item: SearchItem) {
        recentItems.removeAll { $0.id == item.id }
        recentItems.insert(item, at: 0)
        if recentItems.count > Self.maxRecents {
            recentItems.removeLast(recentItems.count - Self.maxRecents)
        }
    }

    func toggleFilter(_ filter: SearchFilter) {
        if activeFilters.contains(filter) {
            activeFilters.remove(filter)
        } else {
            activeFilters.insert(filter)
        }
        performSearch()
    }

    func performSearch() {
        currentWork?.cancel()
        currentWork = nil

        guard !query.isEmpty else {
            results = []
            return
        }

        let q = query.lowercased()
        let qUTF8 = ContiguousArray(q.utf8)
        guard !qUTF8.isEmpty else {
            results = []
            return
        }

        let idx = searchIndex
        let filters = activeFilters

        if idx.items.count <= Self.syncThreshold {
            results = Self.score(query: qUTF8, index: idx, filters: filters)
        } else {
            let work = DispatchWorkItem { [weak self] in
                let items = Self.score(query: qUTF8, index: idx, filters: filters)
                DispatchQueue.main.async { [weak self] in
                    self?.results = items
                }
            }
            currentWork = work
            searchQueue.async(execute: work)
        }
    }
}
