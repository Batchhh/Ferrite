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
