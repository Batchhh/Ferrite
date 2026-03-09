import SwiftUI
import AppKit

// MARK: - Title Bar Button

struct TitleBarButton: View {
    let icon: String
    let action: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(isHovered ? .primary : .secondary)
                .frame(width: 30, height: 30)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(.white.opacity(isHovered ? 0.08 : 0))
                )
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
            if hovering { NSCursor.pointingHand.set() } else { NSCursor.arrow.set() }
        }
        .animation(.easeInOut(duration: 0.15), value: isHovered)
    }
}

// MARK: - Language Toggle Button

struct LanguageToggleButton: View {
    let language: CodeLanguage
    let action: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: action) {
            Text(language.rawValue)
                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                .foregroundStyle(isHovered ? .primary : .secondary)
                .contentTransition(.numericText())
                .animation(.easeInOut(duration: 0.2), value: language)
                .frame(width: 30, height: 30)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(.white.opacity(isHovered ? 0.08 : 0))
                )
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
            if hovering { NSCursor.pointingHand.set() } else { NSCursor.arrow.set() }
        }
        .animation(.easeInOut(duration: 0.15), value: isHovered)
        .help("Switch to \(language == .csharp ? "IL" : "C#") (⌘L)")
    }
}

/// Transparent area that allows dragging the window, replacing the native title bar drag behavior.
struct WindowDragArea: NSViewRepresentable {
    func makeNSView(context: Context) -> DragView { DragView() }
    func updateNSView(_ nsView: DragView, context: Context) {}

    class DragView: NSView {
        override func mouseDown(with event: NSEvent) {
            window?.performDrag(with: event)
        }

        override func mouseUp(with event: NSEvent) {
            // Double-click to zoom
            if event.clickCount == 2 {
                window?.zoom(nil)
            }
        }

        override var intrinsicContentSize: NSSize {
            NSSize(width: NSView.noIntrinsicMetric, height: NSView.noIntrinsicMetric)
        }
    }

    func sizeThatFits(_ proposal: ProposedViewSize, nsView: DragView, context: Context) -> CGSize? {
        // Fill available space — acts like Spacer
        CGSize(
            width: proposal.width ?? 0,
            height: proposal.height ?? 0
        )
    }
}

// MARK: - Flow Layout

struct FlowLayout: Layout {
    var spacing: CGFloat = 6

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = arrange(proposal: proposal, subviews: subviews)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = arrange(proposal: proposal, subviews: subviews)
        for (index, subview) in subviews.enumerated() {
            subview.place(
                at: CGPoint(x: bounds.minX + result.positions[index].x,
                             y: bounds.minY + result.positions[index].y),
                proposal: .unspecified
            )
        }
    }

    private func arrange(proposal: ProposedViewSize, subviews: Subviews) -> (positions: [CGPoint], size: CGSize) {
        let maxWidth = proposal.width ?? .infinity
        var positions: [CGPoint] = []
        var x: CGFloat = 0
        var y: CGFloat = 0
        var rowHeight: CGFloat = 0
        var maxX: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if x + size.width > maxWidth && x > 0 {
                x = 0
                y += rowHeight + spacing
                rowHeight = 0
            }
            positions.append(CGPoint(x: x, y: y))
            rowHeight = max(rowHeight, size.height)
            x += size.width + spacing
            maxX = max(maxX, x - spacing)
        }

        return (positions, CGSize(width: maxX, height: y + rowHeight))
    }
}

// MARK: - Tag Chip

struct TagChip: View {
    let tag: ProjectTag

    var body: some View {
        Text(tag.name)
            .font(.caption2.weight(.medium))
            .foregroundStyle(tag.color.color)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(
                Capsule()
                    .fill(tag.color.color.opacity(0.15))
            )
    }
}
