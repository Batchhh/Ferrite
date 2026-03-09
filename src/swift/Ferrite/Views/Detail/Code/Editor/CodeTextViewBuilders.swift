import SwiftUI
import AppKit

// MARK: - Code View Builders

extension CodeView {
    func buildAttributedCode() -> NSAttributedString {
        let result = NSMutableAttributedString()
        let font = NSFont.monospacedSystemFont(ofSize: Self.fontSize, weight: .regular)
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.minimumLineHeight = Self.lineHeightValue
        paragraphStyle.maximumLineHeight = Self.lineHeightValue

        for (index, line) in lines.enumerated() {
            if line.tokens.isEmpty {
                let attrs: [NSAttributedString.Key: Any] = [
                    .font: font,
                    .paragraphStyle: paragraphStyle,
                ]
                result.append(NSAttributedString(string: " ", attributes: attrs))
            } else {
                for token in line.tokens {
                    var attrs: [NSAttributedString.Key: Any] = [
                        .font: font,
                        .foregroundColor: NSColor(token.color),
                        .paragraphStyle: paragraphStyle,
                    ]
                    if let name = token.typeName {
                        if name.hasPrefix("ferrite://prop/") || name.hasPrefix("ferrite://method/") {
                            let url = URL(string: name)!
                            attrs[.link] = url
                            attrs[.underlineStyle] = 0
                        } else if name.hasPrefix("ferrite://member/") {
                            let url = URL(string: name)!
                            attrs[.link] = url
                            attrs[.underlineStyle] = 0
                        } else if let resolved = resolveType?(name) {
                            let url = URL(string: "ferrite://type/\(resolved.assemblyId)/\(resolved.token)")!
                            attrs[.link] = url
                            attrs[.underlineStyle] = 0
                        }
                    }
                    result.append(NSAttributedString(string: token.text, attributes: attrs))
                }
            }
            if index < lines.count - 1 {
                result.append(NSAttributedString(string: "\n"))
            }
        }
        return result
    }

    func buildLineNumbers() -> NSAttributedString {
        let result = NSMutableAttributedString()
        let font = NSFont.monospacedSystemFont(ofSize: Self.fontSize, weight: .regular)
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.minimumLineHeight = Self.lineHeightValue
        paragraphStyle.maximumLineHeight = Self.lineHeightValue
        paragraphStyle.alignment = .right
        paragraphStyle.tailIndent = -8  // 8pt right padding

        let attrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: NSColor(white: 1.0, alpha: 0.22),
            .paragraphStyle: paragraphStyle,
        ]

        for i in 0..<max(1, lines.count) {
            if i > 0 { result.append(NSAttributedString(string: "\n")) }
            result.append(NSAttributedString(string: "\(i + 1)", attributes: attrs))
        }
        return result
    }
}

// MARK: - Code Container View

final class CodeContainerView: NSView {
    weak var codeScrollView: NSScrollView?
    weak var gutterScrollView: NSScrollView?
    private var isSyncing = false

    @objc func codeDidScroll(_ notification: Notification) {
        guard !isSyncing, let codeScroll = codeScrollView,
              let gutterScroll = gutterScrollView else { return }
        isSyncing = true
        let codeOrigin = codeScroll.contentView.bounds.origin
        gutterScroll.contentView.scroll(to: NSPoint(x: 0, y: codeOrigin.y))
        gutterScroll.reflectScrolledClipView(gutterScroll.contentView)
        isSyncing = false
    }

    @objc func gutterDidScroll(_ notification: Notification) {
        guard !isSyncing, let codeScroll = codeScrollView,
              let gutterScroll = gutterScrollView else { return }
        isSyncing = true
        let gutterOrigin = gutterScroll.contentView.bounds.origin
        var codeOrigin = codeScroll.contentView.bounds.origin
        codeOrigin.y = gutterOrigin.y
        codeScroll.contentView.scroll(to: codeOrigin)
        codeScroll.reflectScrolledClipView(codeScroll.contentView)
        isSyncing = false
    }
}
