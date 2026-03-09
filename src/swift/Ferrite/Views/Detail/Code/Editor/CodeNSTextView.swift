import SwiftUI
import AppKit

// MARK: - CodeNSTextView

/// `NSTextView` subclass with hover effects and `ferrite://` link handling.
final class CodeNSTextView: NSTextView {
    var onNavigate: ((String, UInt32) -> Void)?
    var onNavigateMember: ((String, UInt32, UInt32) -> Void)?
    var onToggleProperty: ((UInt32) -> Void)?
    var onToggleMethod: ((UInt32) -> Void)?
    private var hoveredLinkRange: NSRange?
    private var hoveredPropRanges: [NSRange] = []
    private var savedPropColors: [(NSRange, NSColor)] = []

    override var acceptsFirstResponder: Bool { true }

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        for area in trackingAreas { removeTrackingArea(area) }
        addTrackingArea(NSTrackingArea(
            rect: bounds,
            options: [.mouseMoved, .mouseEnteredAndExited, .activeInActiveApp, .inVisibleRect],
            owner: self
        ))
    }

    override func mouseMoved(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        let charIndex = characterIndexForInsertion(at: point)
        guard let storage = textStorage, charIndex < storage.length else {
            clearHover()
            super.mouseMoved(with: event)
            return
        }

        var effectiveRange = NSRange(location: 0, length: 0)
        let attrs = storage.attributes(at: charIndex, effectiveRange: &effectiveRange)

        if let link = attrs[.link] as? URL {
            if link.host == "prop" || link.host == "method" {
                let propRanges = findAllRanges(for: link, in: storage)
                if propRanges != hoveredPropRanges {
                    clearHover()
                    hoveredPropRanges = propRanges
                    let brightColor = NSColor(Color.codePlain)
                    for range in propRanges {
                        if let origColor = storage.attribute(.foregroundColor, at: range.location, effectiveRange: nil) as? NSColor {
                            savedPropColors.append((range, origColor))
                        }
                        storage.addAttribute(.foregroundColor, value: brightColor, range: range)
                    }
                }
            } else {
                if hoveredLinkRange != effectiveRange {
                    clearHover()
                    hoveredLinkRange = effectiveRange
                    storage.addAttribute(.underlineStyle, value: NSUnderlineStyle.single.rawValue, range: effectiveRange)
                }
            }
        } else {
            clearHover()
        }

        super.mouseMoved(with: event)
    }

    override func mouseExited(with event: NSEvent) {
        clearHover()
        super.mouseExited(with: event)
    }

    func clearHover() {
        if let range = hoveredLinkRange {
            textStorage?.removeAttribute(.underlineStyle, range: range)
            hoveredLinkRange = nil
        }
        if !hoveredPropRanges.isEmpty {
            for (range, color) in savedPropColors {
                textStorage?.addAttribute(.foregroundColor, value: color, range: range)
            }
            hoveredPropRanges = []
            savedPropColors = []
        }
    }

    /// Find all character ranges in the text storage that have the same link URL.
    private func findAllRanges(for url: URL, in storage: NSTextStorage) -> [NSRange] {
        var ranges: [NSRange] = []
        var pos = 0
        while pos < storage.length {
            var effectiveRange = NSRange(location: 0, length: 0)
            let attrs = storage.attributes(at: pos, effectiveRange: &effectiveRange)
            if let link = attrs[.link] as? URL, link == url {
                if let last = ranges.last, last.location + last.length == effectiveRange.location {
                    // Merge adjacent ranges
                    ranges[ranges.count - 1] = NSRange(location: last.location, length: last.length + effectiveRange.length)
                } else {
                    ranges.append(effectiveRange)
                }
            }
            pos = effectiveRange.location + effectiveRange.length
        }
        return ranges
    }

    override func mouseDown(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        let charIndex = characterIndexForInsertion(at: point)
        guard let storage = textStorage, charIndex < storage.length else {
            super.mouseDown(with: event)
            return
        }

        var effectiveRange = NSRange(location: 0, length: 0)
        let attrs = storage.attributes(at: charIndex, effectiveRange: &effectiveRange)

        if let url = attrs[.link] as? URL, url.scheme == "ferrite" {
            handleFerriteLink(url)
            return
        }

        super.mouseDown(with: event)
    }

    override func clicked(onLink link: Any, at charIndex: Int) {
        guard let url = link as? URL, url.scheme == "ferrite" else {
            super.clicked(onLink: link, at: charIndex)
            return
        }
        handleFerriteLink(url)
    }

    private func handleFerriteLink(_ url: URL) {
        if url.host == "prop" {
            // URL format: ferrite://prop/{token}
            let components = url.pathComponents.filter { $0 != "/" }
            guard components.count == 1, let token = UInt32(components[0]) else { return }
            onToggleProperty?(token)
        } else if url.host == "method" {
            // URL format: ferrite://method/{token}
            let components = url.pathComponents.filter { $0 != "/" }
            guard components.count == 1, let token = UInt32(components[0]) else { return }
            onToggleMethod?(token)
        } else if url.host == "member" {
            // URL format: ferrite://member/{assemblyId}/{typeToken}/{memberToken}
            let components = url.pathComponents.filter { $0 != "/" }
            guard components.count == 3,
                  let typeToken = UInt32(components[1]),
                  let memberToken = UInt32(components[2]) else { return }
            onNavigateMember?(components[0], typeToken, memberToken)
        } else if url.host == "type" {
            // URL format: ferrite://type/{assemblyId}/{token}
            let components = url.pathComponents.filter { $0 != "/" }
            guard components.count == 2, let token = UInt32(components[1]) else { return }
            onNavigate?(components[0], token)
        }
    }
}
