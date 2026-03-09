import Dispatch

// MARK: - Scoring (pure, no side effects)

extension SearchService {
    /// Score all candidates against the query and return the top results.
    static func score(
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
