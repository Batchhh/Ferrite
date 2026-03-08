import SwiftUI

func iconForTypeKind(_ kind: TypeKind) -> String {
    switch kind {
    case .`class`: return "cube"
    case .interface: return "diamond"
    case .`struct`: return "square.on.square"
    case .`enum`: return "tag"
    case .delegate: return "arrowshape.forward"
    }
}

func iconForMemberKind(_ kind: MemberKind) -> String {
    switch kind {
    case .method: return "m.square"
    case .field: return "f.square"
    case .property: return "p.square"
    case .event: return "bolt.fill"
    }
}

func colorForTypeKind(_ kind: TypeKind) -> Color {
    switch kind {
    case .`class`: return Color(NSColor.controlAccentColor)
    case .interface: return .orange
    case .`struct`: return .green
    case .`enum`: return .purple
    case .delegate: return .teal
    }
}

func colorForMemberKind(_ kind: MemberKind) -> Color {
    switch kind {
    case .method: return Color(NSColor.controlAccentColor)
    case .field: return .purple
    case .property: return .green
    case .event: return .orange
    }
}

func visibilityLabel(_ visibility: Visibility) -> String {
    switch visibility {
    case .`public`: return "public"
    case .`private`: return "private"
    case .`internal`: return "internal"
    case .protected: return "protected"
    case .protectedInternal: return "protected internal"
    case .privateProtected: return "private protected"
    }
}

func typeKindLabel(_ kind: TypeKind) -> String {
    switch kind {
    case .`class`: return "class"
    case .interface: return "interface"
    case .`struct`: return "struct"
    case .`enum`: return "enum"
    case .delegate: return "delegate"
    }
}

func memberKindLabel(_ kind: MemberKind) -> String {
    switch kind {
    case .method: return "method"
    case .field: return "field"
    case .property: return "property"
    case .event: return "event"
    }
}
