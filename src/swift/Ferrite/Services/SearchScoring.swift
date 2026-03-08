import Foundation

// MARK: - Scoring

extension SearchIndex {
    /// Score a candidate against a query using UTF-8 byte comparison.
    ///
    /// Scoring tiers (higher is better):
    /// - 10 000: exact title match
    /// -  5 000: title prefix match (boosted by query/title length ratio)
    /// -  2 000: title contains query (boosted by query/title length ratio)
    /// -    500: subtitle contains query
    /// -      0: no match
    @inline(__always)
    static func scoreFlat(
        title: ContiguousArray<UInt8>,
        subtitle: ContiguousArray<UInt8>,
        query: ContiguousArray<UInt8>
    ) -> Int {
        let qCount = query.count
        let tCount = title.count

        if tCount == qCount && title.elementsEqual(query) { return 10000 }

        if tCount >= qCount {
            var isPrefix = true
            for j in 0..<qCount {
                if title[j] != query[j] { isPrefix = false; break }
            }
            if isPrefix { return 5000 + (qCount * 100) / max(tCount, 1) }
        }

        if utf8Contains(title, query) {
            return 2000 + (qCount * 100) / max(tCount, 1)
        }

        if utf8Contains(subtitle, query) { return 500 }

        return 0
    }

    /// Return `true` if `haystack` contains `needle` as a contiguous subsequence.
    @inline(__always)
    static func utf8Contains(_ haystack: ContiguousArray<UInt8>, _ needle: ContiguousArray<UInt8>) -> Bool {
        let nCount = needle.count
        let hCount = haystack.count
        guard nCount <= hCount else { return false }
        if nCount == 0 { return true }
        let limit = hCount - nCount
        let first = needle[0]
        for i in 0...limit {
            if haystack[i] == first {
                var match = true
                for j in 1..<nCount {
                    if haystack[i &+ j] != needle[j] { match = false; break }
                }
                if match { return true }
            }
        }
        return false
    }
}
