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

    static let fontSize: CGFloat = 12
    static let lineHeightValue: CGFloat = 20
    static let gutterWidth: CGFloat = 48

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
        gutterTextView.textContainer?.containerSize = NSSize(
            width: Self.gutterWidth,
            height: CGFloat.greatestFiniteMagnitude
        )
        gutterTextView.minSize = NSSize(width: Self.gutterWidth, height: 0)
        gutterTextView.maxSize = NSSize(width: Self.gutterWidth, height: CGFloat.greatestFiniteMagnitude)
        gutterTextView.isVerticallyResizable = true
        gutterTextView.isHorizontallyResizable = false
        gutterTextView.autoresizingMask = [.width]

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
        updateDocumentFrames(
            textView: textView,
            gutterTextView: gutterTextView,
            codeScroll: codeScroll,
            gutterScroll: gutterScroll
        )
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
        updateDocumentFrames(
            textView: textView,
            gutterTextView: gutterTextView,
            codeScroll: codeScroll,
            gutterScroll: gutterScroll
        )
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

    private func updateDocumentFrames(
        textView: NSTextView,
        gutterTextView: NSTextView,
        codeScroll: NSScrollView,
        gutterScroll: NSScrollView
    ) {
        let codeSize = documentSize(for: textView)
        let gutterSize = documentSize(for: gutterTextView)
        let contentHeight = max(codeSize.height, gutterSize.height, codeScroll.contentSize.height, gutterScroll.contentSize.height)

        textView.setFrameSize(NSSize(
            width: max(codeSize.width, codeScroll.contentSize.width),
            height: contentHeight
        ))
        gutterTextView.setFrameSize(NSSize(
            width: Self.gutterWidth,
            height: contentHeight
        ))
    }

    private func documentSize(for textView: NSTextView) -> NSSize {
        guard let layoutManager = textView.layoutManager,
              let textContainer = textView.textContainer else {
            return textView.fittingSize
        }

        layoutManager.ensureLayout(for: textContainer)
        var usedRect = layoutManager.usedRect(for: textContainer)
        usedRect.size.width += textView.textContainerInset.width * 2
        usedRect.size.height += textView.textContainerInset.height * 2
        return usedRect.integral.size
    }
}
