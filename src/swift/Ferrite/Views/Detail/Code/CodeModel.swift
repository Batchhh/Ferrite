import SwiftUI

// MARK: - Code Model

/// A single syntax-colored token within a line of C# code.
struct CodeToken {
    let text: String
    let color: Color
    let typeName: String?

    init(text: String, color: Color, typeName: String? = nil) {
        self.text = text
        self.color = color
        self.typeName = typeName
    }

    static func keyword(_ t: String)     -> Self { .init(text: t, color: .codeKeyword) }
    static func controlFlow(_ t: String) -> Self { .init(text: t, color: .codeControlFlow) }
    static func type_(_ t: String)       -> Self { .init(text: t, color: .codeType, typeName: t) }
    static func method(_ t: String)      -> Self { .init(text: t, color: .codeMethod) }
    static func field(_ t: String)       -> Self { .init(text: t, color: .codeField) }
    static func comment(_ t: String)     -> Self { .init(text: t, color: .codeComment) }
    static func plain(_ t: String)       -> Self { .init(text: t, color: .codePlain) }
    static func punct(_ t: String)       -> Self { .init(text: t, color: .codePlain) }
    static func string(_ t: String)      -> Self { .init(text: t, color: .codeString) }
    static func number(_ t: String)      -> Self { .init(text: t, color: .codeNumber) }
    static let space = CodeToken(text: " ", color: .codePlain)
}

/// A tokenized line of C# code rendered in the code view.
struct CodeLine {
    let tokens: [CodeToken]
    static let empty = CodeLine(tokens: [])

    func toAttributedString() -> AttributedString {
        if tokens.isEmpty { return AttributedString(" ") }
        var result = AttributedString()
        for token in tokens {
            var part = AttributedString(token.text)
            part.foregroundColor = NSColor(token.color)
            result += part
        }
        return result
    }
}
