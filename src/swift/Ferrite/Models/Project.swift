import Foundation
import SwiftUI

enum TagColor: String, Codable, CaseIterable {
    case red, orange, yellow, green, blue, purple, pink, gray

    var color: Color {
        switch self {
        case .red: .red
        case .orange: .orange
        case .yellow: .yellow
        case .green: .green
        case .blue: .blue
        case .purple: .purple
        case .pink: .pink
        case .gray: .gray
        }
    }
}

/// A user-defined label that can be applied to projects for filtering.
struct ProjectTag: Codable, Identifiable, Hashable {
    var id: UUID = UUID()
    var name: String
    var color: TagColor
}

/// A saved workspace: a named collection of .NET assembly paths and associated tags.
struct Project: Codable, Identifiable, Hashable {
    var id: UUID = UUID()
    var name: String
    var dllPaths: [String] = []
    /// IDs of `ProjectTag` values stored separately in `ProjectService.availableTags`.
    var tags: [UUID] = []
    var createdAt: Date = Date()
    var lastOpenedAt: Date = Date()
    /// Per-item tags: storageKey → array of ItemTag raw values.
    var itemTags: [String: [String]] = [:]
}
