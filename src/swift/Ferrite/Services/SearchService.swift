import Dispatch
import SwiftUI

/// Main-actor observable service managing search state, query debouncing, and index rebuilding.
/// For small-to-medium indexes (≤30k items), scoring runs synchronously on the main thread
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
    private let searchQueue = DispatchQueue(label: "ferrite.search", qos: .userInteractive)

    /// Current in-flight search work item — cancelled when a new search starts.
    private var currentWork: DispatchWorkItem?

    /// Debounce work item for the async path.
    private var debounceWork: DispatchWorkItem?

    /// Indexes at or below this size are scored synchronously on the main thread.
    private static let syncThreshold = 30_000

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
            // Sync path: no debounce needed, results appear same frame
            performSearch()
        } else {
            // Async path: debounce at 30ms to coalesce rapid keystrokes
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
        // Cancel any in-flight async work
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
            // SYNC path: score directly on main thread, results appear same frame
            results = Self.score(query: qUTF8, index: idx, filters: filters)
        } else {
            // ASYNC path: dispatch to background with cancellation
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

    // MARK: - Scoring (pure, no side effects)

    /// Score all candidates against the query and return the top results.
    private static func score(
        query qUTF8: ContiguousArray<UInt8>,
        index idx: SearchIndex,
        filters: Set<SearchFilter>,
        maxResults: Int = 50
    ) -> [SearchItem] {
        guard let firstByte = qUTF8.first else { return [] }

        let titles = idx.titleBytes
        let subtitles = idx.subtitleBytes

        // Determine candidate indices from prefix buckets
        let prefixIndices: ContiguousArray<Int>?
        if filters.isEmpty {
            prefixIndices = idx.prefixBuckets[firstByte]
        } else if filters.count == 1, let single = filters.first {
            prefixIndices = idx.kindPrefixBuckets[single]?[firstByte]
        } else {
            var merged = ContiguousArray<Int>()
            for filter in filters {
                if let bucket = idx.kindPrefixBuckets[filter]?[firstByte] {
                    merged.append(contentsOf: bucket)
                }
            }
            prefixIndices = merged.isEmpty ? nil : merged
        }

        // Score prefix-bucket items using flat arrays (no SearchItem struct copying)
        var scored: [(index: Int, score: Int)] = []
        scored.reserveCapacity(min(prefixIndices?.count ?? 0, 256))

        if let bucket = prefixIndices {
            for i in bucket {
                let score = SearchIndex.scoreFlat(title: titles[i], subtitle: subtitles[i], query: qUTF8)
                if score > 0 { scored.append((i, score)) }
            }
        }

        // Scan non-prefix ("contains") matches if needed, with a cap to prevent worst-case scans
        if scored.count < maxResults {
            let prefixSet = Set(prefixIndices ?? [])
            let hasFilters = !filters.isEmpty
            let scanLimit = 5000

            if hasFilters {
                var scanned = 0
                outer: for filter in filters {
                    if let kindIndices = idx.indexByKind[filter] {
                        for i in kindIndices {
                            if prefixSet.contains(i) { continue }
                            let score = SearchIndex.scoreFlat(title: titles[i], subtitle: subtitles[i], query: qUTF8)
                            if score > 0 {
                                scored.append((i, score))
                                if scored.count >= maxResults * 2 { break outer }
                            }
                            scanned += 1
                            if scanned >= scanLimit { break outer }
                        }
                    }
                }
            } else {
                var scanned = 0
                for i in titles.indices {
                    if prefixSet.contains(i) { continue }
                    let score = SearchIndex.scoreFlat(title: titles[i], subtitle: subtitles[i], query: qUTF8)
                    if score > 0 {
                        scored.append((i, score))
                        if scored.count >= maxResults * 2 { break }
                    }
                    scanned += 1
                    if scanned >= scanLimit { break }
                }
            }
        }

        scored.sort { $0.score > $1.score }
        let limit = min(scored.count, maxResults)
        return (0..<limit).map { idx.items[scored[$0].index] }
    }
}
