import SwiftUI

// MARK: - Index Building

extension SearchIndex {
    /// Build a `SearchIndex` from searchable items fetched per assembly.
    static func build(from assemblies: [AssemblySummary], session: DecompilerSession) -> SearchIndex {
        var items = ContiguousArray<SearchItem>()

        for entry in assemblies {
            let asmName = entry.name
            guard let searchableItems = try? session.getSearchableItems(assemblyId: entry.id) else { continue }
            items.reserveCapacity(items.count + searchableItems.count)

            for si in searchableItems {
                let titleLower = si.name.lowercased()
                let titleUTF8 = ContiguousArray(titleLower.utf8)
                let subtitleLower = ContiguousArray(si.fullName.lowercased().utf8)
                let filter = filterForSearchableKind(si.kind)

                let selection: Selection
                if let parentToken = si.parentToken {
                    selection = .member(assemblyId: entry.id, typeToken: parentToken, memberToken: si.token)
                } else {
                    selection = .type(assemblyId: entry.id, token: si.token)
                }

                items.append(SearchItem(
                    id: "\(entry.id):\(si.token)",
                    title: si.name,
                    subtitle: si.fullName,
                    assemblyName: asmName,
                    selection: selection,
                    kind: filter,
                    titleUTF8: titleUTF8,
                    subtitleUTF8: subtitleLower,
                    firstByte: titleUTF8.first ?? 0
                ))
            }
        }

        // Build all lookup tables in one pass
        var buckets: [UInt8: ContiguousArray<Int>] = [:]
        var byKind: [SearchFilter: ContiguousArray<Int>] = [:]
        for filter in SearchFilter.allCases {
            byKind[filter] = ContiguousArray<Int>()
        }
        for i in items.indices {
            let item = items[i]
            buckets[item.firstByte, default: ContiguousArray<Int>()].append(i)
            byKind[item.kind]!.append(i)
        }

        var kpBuckets: [SearchFilter: [UInt8: ContiguousArray<Int>]] = [:]
        for (kind, indices) in byKind {
            var kindBuckets: [UInt8: ContiguousArray<Int>] = [:]
            for i in indices {
                kindBuckets[items[i].firstByte, default: ContiguousArray<Int>()].append(i)
            }
            kpBuckets[kind] = kindBuckets
        }

        var titleBytes = ContiguousArray<ContiguousArray<UInt8>>()
        var subtitleBytes = ContiguousArray<ContiguousArray<UInt8>>()
        var firstByteArr = ContiguousArray<UInt8>()
        titleBytes.reserveCapacity(items.count)
        subtitleBytes.reserveCapacity(items.count)
        firstByteArr.reserveCapacity(items.count)
        for i in items.indices {
            titleBytes.append(items[i].titleUTF8)
            subtitleBytes.append(items[i].subtitleUTF8)
            firstByteArr.append(items[i].firstByte)
        }

        return SearchIndex(
            items: items, titleBytes: titleBytes, subtitleBytes: subtitleBytes,
            firstBytes: firstByteArr, prefixBuckets: buckets, indexByKind: byKind,
            kindPrefixBuckets: kpBuckets
        )
    }

    private static func filterForSearchableKind(_ kind: SearchableKind) -> SearchFilter {
        switch kind {
        case .`class`: return .class_
        case .interface: return .interface
        case .`struct`: return .struct_
        case .`enum`: return .enum_
        case .delegate: return .delegate
        case .method: return .method
        case .field: return .field
        case .property: return .property
        case .constant: return .constant
        }
    }
}
