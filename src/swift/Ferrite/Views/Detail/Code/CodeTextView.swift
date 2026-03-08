import SwiftUI
import AppKit

// MARK: - Code View

struct CodeView: NSViewRepresentable {
    let lines: [CodeLine]
    var searchQuery: String = ""
    var currentMatchIndex: Int = 0
    var matchRanges: Binding<[NSRange]>? = nil
    var resolveType: ((String) -> (assemblyId: String, token: UInt32)?)? = nil
    var onNavigate: ((String, UInt32) -> Void)? = nil
    var onNavigateMember: ((String, UInt32, UInt32) -> Void)? = nil
    var onToggleProperty: ((UInt32) -> Void)? = nil
    var onToggleMethod: ((UInt32) -> Void)? = nil

    private static let fontSize: CGFloat = 12
    private static let lineHeightValue: CGFloat = 20
    private static let gutterWidth: CGFloat = 48

    func makeNSView(context: Context) -> CodeContainerView {
        let container = CodeContainerView()

        let textView = CodeNSTextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.drawsBackground = false
        textView.isRichText = true
        textView.textContainerInset = NSSize(width: 16, height: 12)
        textView.textContainer?.lineFragmentPadding = 0
        textView.textContainer?.widthTracksTextView = false
        textView.textContainer?.containerSize = NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        textView.minSize = NSSize(width: 0, height: 0)
        textView.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        textView.isHorizontallyResizable = true
        textView.isVerticallyResizable = true
        textView.autoresizingMask = [.width]
        textView.onNavigate = onNavigate
        textView.onNavigateMember = onNavigateMember
        textView.onToggleProperty = onToggleProperty
        textView.onToggleMethod = onToggleMethod
        textView.linkTextAttributes = [
            .cursor: NSCursor.pointingHand,
        ]

        let codeScroll = NSScrollView()
        codeScroll.documentView = textView
        codeScroll.hasVerticalScroller = false
        codeScroll.hasHorizontalScroller = false
        codeScroll.drawsBackground = false
        codeScroll.translatesAutoresizingMaskIntoConstraints = false

        let gutterTextView = NSTextView()
        gutterTextView.isEditable = false
        gutterTextView.isSelectable = false
        gutterTextView.drawsBackground = false
        gutterTextView.isRichText = true
        gutterTextView.textContainerInset = NSSize(width: 0, height: 12)
        gutterTextView.textContainer?.lineFragmentPadding = 0
        gutterTextView.textContainer?.widthTracksTextView = true
        gutterTextView.isVerticallyResizable = true
        gutterTextView.isHorizontallyResizable = false

        let gutterScroll = NSScrollView()
        gutterScroll.documentView = gutterTextView
        gutterScroll.hasVerticalScroller = false
        gutterScroll.hasHorizontalScroller = false
        gutterScroll.drawsBackground = false
        gutterScroll.translatesAutoresizingMaskIntoConstraints = false

        container.addSubview(gutterScroll)
        container.addSubview(codeScroll)

        NSLayoutConstraint.activate([
            gutterScroll.leadingAnchor.constraint(equalTo: container.leadingAnchor),
            gutterScroll.topAnchor.constraint(equalTo: container.topAnchor),
            gutterScroll.bottomAnchor.constraint(equalTo: container.bottomAnchor),
            gutterScroll.widthAnchor.constraint(equalToConstant: Self.gutterWidth),

            codeScroll.leadingAnchor.constraint(equalTo: gutterScroll.trailingAnchor),
            codeScroll.topAnchor.constraint(equalTo: container.topAnchor),
            codeScroll.bottomAnchor.constraint(equalTo: container.bottomAnchor),
            codeScroll.trailingAnchor.constraint(equalTo: container.trailingAnchor),
        ])

        container.codeScrollView = codeScroll
        container.gutterScrollView = gutterScroll

        // Sync vertical scrolling between code and gutter
        codeScroll.contentView.postsBoundsChangedNotifications = true
        gutterScroll.contentView.postsBoundsChangedNotifications = true
        NotificationCenter.default.addObserver(
            container, selector: #selector(CodeContainerView.codeDidScroll(_:)),
            name: NSView.boundsDidChangeNotification,
            object: codeScroll.contentView
        )
        NotificationCenter.default.addObserver(
            container, selector: #selector(CodeContainerView.gutterDidScroll(_:)),
            name: NSView.boundsDidChangeNotification,
            object: gutterScroll.contentView
        )

        textView.textStorage?.setAttributedString(buildAttributedCode())
        gutterTextView.textStorage?.setAttributedString(buildLineNumbers())
        if let lm = textView.layoutManager, let tc = textView.textContainer {
            lm.ensureLayout(for: tc)
        }
        applySearchHighlights(textView: textView)
        textView.scrollToBeginningOfDocument(nil)
        return container
    }

    func updateNSView(_ container: CodeContainerView, context: Context) {
        guard let codeScroll = container.codeScrollView,
              let textView = codeScroll.documentView as? CodeNSTextView,
              let gutterScroll = container.gutterScrollView,
              let gutterTextView = gutterScroll.documentView as? NSTextView else { return }
        textView.clearHover()
        textView.onNavigate = onNavigate
        textView.onNavigateMember = onNavigateMember
        textView.onToggleProperty = onToggleProperty
        textView.onToggleMethod = onToggleMethod
        let savedOrigin = codeScroll.contentView.bounds.origin
        textView.textStorage?.setAttributedString(buildAttributedCode())
        gutterTextView.textStorage?.setAttributedString(buildLineNumbers())
        // Force layout recalculation so the scroll view knows the new document size
        if let layoutManager = textView.layoutManager, let textContainer = textView.textContainer {
            layoutManager.ensureLayout(for: textContainer)
        }
        if let gutterLayout = gutterTextView.layoutManager, let gutterContainer = gutterTextView.textContainer {
            gutterLayout.ensureLayout(for: gutterContainer)
        }
        codeScroll.contentView.scroll(to: savedOrigin)
        codeScroll.reflectScrolledClipView(codeScroll.contentView)
        gutterScroll.contentView.scroll(to: NSPoint(x: 0, y: savedOrigin.y))
        gutterScroll.reflectScrolledClipView(gutterScroll.contentView)

        applySearchHighlights(textView: textView)
    }

    private func applySearchHighlights(textView: NSTextView) {
        guard let layoutManager = textView.layoutManager,
              let textContainer = textView.textContainer else { return }
        let fullRange = NSRange(location: 0, length: (textView.string as NSString).length)
        layoutManager.removeTemporaryAttribute(.backgroundColor, forCharacterRange: fullRange)

        guard !searchQuery.isEmpty else {
            DispatchQueue.main.async { self.matchRanges?.wrappedValue = [] }
            return
        }

        let text = textView.string as NSString
        var ranges: [NSRange] = []
        var searchRange = NSRange(location: 0, length: text.length)
        while searchRange.location < text.length {
            let found = text.range(of: searchQuery, options: .caseInsensitive, range: searchRange)
            if found.location == NSNotFound { break }
            ranges.append(found)
            searchRange.location = found.location + found.length
            searchRange.length = text.length - searchRange.location
        }

        DispatchQueue.main.async { self.matchRanges?.wrappedValue = ranges }

        let matchColor = NSColor(Color.codeSearchMatch)
        let currentColor = NSColor(Color.codeSearchCurrentMatch)

        for range in ranges {
            layoutManager.addTemporaryAttribute(.backgroundColor, value: matchColor, forCharacterRange: range)
        }

        if !ranges.isEmpty {
            let idx = min(currentMatchIndex, ranges.count - 1)
            let currentRange = ranges[idx]
            layoutManager.addTemporaryAttribute(.backgroundColor, value: currentColor, forCharacterRange: currentRange)

            let glyphRange = layoutManager.glyphRange(forCharacterRange: currentRange, actualCharacterRange: nil)
            let rect = layoutManager.boundingRect(forGlyphRange: glyphRange, in: textContainer)
            let scrollRect = rect.insetBy(dx: 0, dy: -40).offsetBy(dx: textView.textContainerInset.width, dy: textView.textContainerInset.height)
            textView.scrollToVisible(scrollRect)
        }
    }

    private func buildAttributedCode() -> NSAttributedString {
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

    private func buildLineNumbers() -> NSAttributedString {
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

