import Observation
import SwiftUI

/// Manages per-item tags and sidebar filtering.
@MainActor
@Observable
final class ItemTagService {
    /// All tagged items: storageKey → set of tags.
    private(set) var itemTags: [String: Set<ItemTag>] = [:]

    /// Currently active filter tags.
    var activeFilters: Set<ItemTag> = [] {
        didSet { recomputeVisibility() }
    }

    /// Precomputed set of storageKeys that should be visible when filtering.
    private(set) var visibleKeys: Set<String> = []

    /// Keys of items directly tagged with an active filter (children should show).
    private(set) var taggedAncestorKeys: Set<String> = []

    var isFiltering: Bool { !activeFilters.isEmpty }

    // MARK: - Tag Operations

    func tags(for selection: Selection) -> Set<ItemTag> {
        itemTags[selection.storageKey] ?? []
    }

    func toggleTag(_ tag: ItemTag, on selection: Selection) {
        let key = selection.storageKey
        var current = itemTags[key] ?? []
        if current.contains(tag) {
            current.remove(tag)
        } else {
            current.insert(tag)
        }
        if current.isEmpty {
            itemTags.removeValue(forKey: key)
        } else {
            itemTags[key] = current
        }
        recomputeVisibility()
    }

    // MARK: - Filter Operations

    func toggleFilter(_ tag: ItemTag) {
        if activeFilters.contains(tag) {
            activeFilters.remove(tag)
        } else {
            activeFilters.insert(tag)
        }
    }

    func clearFilters() {
        activeFilters.removeAll()
    }

    // MARK: - Visibility

    /// Whether a sidebar item should be shown given current filters.
    func shouldShow(_ selection: Selection) -> Bool {
        guard isFiltering else { return true }
        let key = selection.storageKey
        // Item itself or an ancestor is in the visible set
        if visibleKeys.contains(key) { return true }
        // Item is a child of a directly tagged ancestor
        for ancestorKey in selection.ancestorKeys {
            if taggedAncestorKeys.contains(ancestorKey) { return true }
        }
        return false
    }

    /// Whether a namespace should be shown (needs to check if any child type is tagged).
    func shouldShowNamespace(assemblyId: String, namespace: String, typeTokens: [UInt32]) -> Bool {
        guard isFiltering else { return true }
        let nsKey = Selection.namespace(assemblyId: assemblyId, name: namespace).storageKey
        if visibleKeys.contains(nsKey) { return true }
        // Check if the assembly ancestor is directly tagged
        let asmKey = Selection.assembly(id: assemblyId).storageKey
        if taggedAncestorKeys.contains(asmKey) { return true }
        // Check if any child type is visible
        for token in typeTokens {
            let typeKey = Selection.type(assemblyId: assemblyId, token: token).storageKey
            if visibleKeys.contains(typeKey) { return true }
        }
        return false
    }

    private func recomputeVisibility() {
        guard isFiltering else {
            visibleKeys = []
            taggedAncestorKeys = []
            return
        }
        var visible = Set<String>()
        var tagged = Set<String>()
        for (key, tags) in itemTags {
            if !tags.isDisjoint(with: activeFilters) {
                visible.insert(key)
                tagged.insert(key)
                // Add all ancestors so tree path is visible
                if let selection = Self.parseStorageKey(key) {
                    for ancestorKey in selection.ancestorKeys {
                        visible.insert(ancestorKey)
                    }
                }
            }
        }
        visibleKeys = visible
        taggedAncestorKeys = tagged
    }

    // MARK: - Persistence Bridge

    func load(from projectService: ProjectService) {
        itemTags = projectService.loadItemTags()
        recomputeVisibility()
    }

    func save(to projectService: ProjectService) {
        projectService.updateItemTags(itemTags)
    }

    // MARK: - Storage Key Parsing

    private static func parseStorageKey(_ key: String) -> Selection? {
        let parts = key.split(separator: ":", maxSplits: 1)
        guard parts.count == 2 else { return nil }
        let prefix = parts[0]
        let rest = String(parts[1])
        switch prefix {
        case "assembly":
            return .assembly(id: rest)
        case "namespace":
            let nsParts = rest.split(separator: ":", maxSplits: 1)
            guard nsParts.count == 2 else { return nil }
            return .namespace(assemblyId: String(nsParts[0]), name: String(nsParts[1]))
        case "type":
            let tParts = rest.split(separator: ":")
            guard tParts.count == 2, let token = UInt32(tParts[1]) else { return nil }
            return .type(assemblyId: String(tParts[0]), token: token)
        case "member":
            let mParts = rest.split(separator: ":")
            guard mParts.count == 3,
                  let typeToken = UInt32(mParts[1]),
                  let memberToken = UInt32(mParts[2]) else { return nil }
            return .member(assemblyId: String(mParts[0]), typeToken: typeToken, memberToken: memberToken)
        default:
            return nil
        }
    }
}
