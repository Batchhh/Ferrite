import SwiftUI

/// Predefined tags that can be applied to any sidebar item.
enum ItemTag: String, Codable, CaseIterable, Hashable {
    case important
    case warning
    case todo
    case reviewed

    var displayName: String {
        switch self {
        case .important: "Important"
        case .warning: "Warning"
        case .todo: "Todo"
        case .reviewed: "Reviewed"
        }
    }

    var color: Color {
        switch self {
        case .important: .red
        case .warning: .orange
        case .todo: .yellow
        case .reviewed: .green
        }
    }

    var icon: String {
        switch self {
        case .important: "exclamationmark.circle.fill"
        case .warning: "exclamationmark.triangle.fill"
        case .todo: "checklist"
        case .reviewed: "checkmark.circle.fill"
        }
    }
}

// MARK: - Selection Storage Key

extension Selection {
    /// Deterministic string key for JSON dictionary storage.
    var storageKey: String {
        switch self {
        case .assembly(let id):
            "assembly:\(id)"
        case .namespace(let assemblyId, let name):
            "namespace:\(assemblyId):\(name)"
        case .type(let assemblyId, let token):
            "type:\(assemblyId):\(token)"
        case .member(let assemblyId, let typeToken, let memberToken):
            "member:\(assemblyId):\(typeToken):\(memberToken)"
        }
    }

    /// Ancestor keys for filtering (does not include self).
    var ancestorKeys: [String] {
        switch self {
        case .assembly:
            []
        case .namespace(let assemblyId, _):
            [Selection.assembly(id: assemblyId).storageKey]
        case .type(let assemblyId, _):
            [Selection.assembly(id: assemblyId).storageKey]
        case .member(let assemblyId, let typeToken, _):
            [
                Selection.assembly(id: assemblyId).storageKey,
                Selection.type(assemblyId: assemblyId, token: typeToken).storageKey,
            ]
        }
    }
}
