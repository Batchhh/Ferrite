import SwiftUI

// MARK: - Type Name Cache

extension DecompilerService {
    /// Rebuild the type name cache from scratch after an assembly is removed.
    func rebuildTypeNameCache() {
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
            self.error = "Failed to build search index: \(error.localizedDescription)"
        }
    }
}
