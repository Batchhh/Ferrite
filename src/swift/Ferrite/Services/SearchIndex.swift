import SwiftUI

// MARK: - SearchFilter

enum SearchFilter: CaseIterable, Hashable {
    case class_, interface, struct_, enum_, delegate
    case method, field, property, event, constant

    var label: String {
        switch self {
        case .class_: return "Class"
        case .interface: return "Interface"
        case .struct_: return "Struct"
        case .enum_: return "Enum"
        case .delegate: return "Delegate"
        case .method: return "Method"
        case .field: return "Field"
        case .property: return "Property"
        case .event: return "Event"
        case .constant: return "Constant"
        }
    }

    var icon: String {
        switch self {
        case .class_: return "cube"
        case .interface: return "diamond"
        case .struct_: return "square.on.square"
        case .enum_: return "tag"
        case .delegate: return "arrowshape.forward"
        case .method: return "m.square"
        case .field: return "f.square"
        case .property: return "p.square"
        case .event: return "bolt.fill"
        case .constant: return "c.square"
        }
    }
}

// MARK: - SearchItem

struct SearchItem: Identifiable, @unchecked Sendable {
    let id: String
    let title: String
    let subtitle: String
    let assemblyName: String
    let selection: Selection
    let kind: SearchFilter

    // Pre-computed for fast matching
    let titleUTF8: ContiguousArray<UInt8>
    let subtitleUTF8: ContiguousArray<UInt8>
    let firstByte: UInt8

    var icon: String {
        switch kind {
        case .class_: return "cube"
        case .interface: return "diamond"
        case .struct_: return "square.on.square"
        case .enum_: return "tag"
        case .delegate: return "arrowshape.forward"
        case .method: return "m.square"
        case .field: return "f.square"
        case .property: return "p.square"
        case .event: return "bolt.fill"
        case .constant: return "c.square"
        }
    }

    var iconColor: Color {
        switch kind {
        case .class_: return colorForTypeKind(.`class`)
        case .interface: return colorForTypeKind(.interface)
        case .struct_: return colorForTypeKind(.`struct`)
        case .enum_: return colorForTypeKind(.`enum`)
        case .delegate: return colorForTypeKind(.delegate)
        case .method: return colorForMemberKind(.method)
        case .field: return colorForMemberKind(.field)
        case .property: return colorForMemberKind(.property)
        case .event: return colorForMemberKind(.event)
        case .constant: return .orange
        }
    }
}

// MARK: - SearchIndex

/// Immutable snapshot of the search index — built on a background thread, read from any thread.
/// Scoring-hot data is stored in flat parallel arrays to avoid struct copying / ARC in hot loops.
final class SearchIndex: @unchecked Sendable {
    let items: ContiguousArray<SearchItem>

    // Flat parallel arrays for scoring — no struct copy / String retain in hot path
    let titleBytes: ContiguousArray<ContiguousArray<UInt8>>
    let subtitleBytes: ContiguousArray<ContiguousArray<UInt8>>
    let firstBytes: ContiguousArray<UInt8>

    let prefixBuckets: [UInt8: ContiguousArray<Int>]
    let indexByKind: [SearchFilter: ContiguousArray<Int>]
    let kindPrefixBuckets: [SearchFilter: [UInt8: ContiguousArray<Int>]]

    static let empty = SearchIndex(
        items: [], titleBytes: [], subtitleBytes: [], firstBytes: [],
        prefixBuckets: [:], indexByKind: [:], kindPrefixBuckets: [:]
    )

    init(
        items: ContiguousArray<SearchItem>,
        titleBytes: ContiguousArray<ContiguousArray<UInt8>>,
        subtitleBytes: ContiguousArray<ContiguousArray<UInt8>>,
        firstBytes: ContiguousArray<UInt8>,
        prefixBuckets: [UInt8: ContiguousArray<Int>],
        indexByKind: [SearchFilter: ContiguousArray<Int>],
        kindPrefixBuckets: [SearchFilter: [UInt8: ContiguousArray<Int>]]
    ) {
        self.items = items
        self.titleBytes = titleBytes
        self.subtitleBytes = subtitleBytes
        self.firstBytes = firstBytes
        self.prefixBuckets = prefixBuckets
        self.indexByKind = indexByKind
        self.kindPrefixBuckets = kindPrefixBuckets
    }
}

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

                // Build selection: types select by token, members select by parent+token
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

