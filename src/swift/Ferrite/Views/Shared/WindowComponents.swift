import SwiftUI
import AppKit

// MARK: - Key Cap

struct KeyCapView: View {
    let key: String

    init(_ key: String) {
        self.key = key
    }

    var body: some View {
        Text(key)
            .font(.system(size: 11, weight: .medium, design: .rounded))
            .foregroundStyle(.secondary)
            .frame(minWidth: 22, minHeight: 20)
            .padding(.horizontal, 4)
            .background(
                RoundedRectangle(cornerRadius: 5)
                    .fill(.white.opacity(0.06))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 5)
                    .strokeBorder(.white.opacity(0.1), lineWidth: 0.5)
            )
    }
}

// MARK: - Window Configurator

/// Configures window chrome — removes separator and tracks fullscreen state.
struct WindowConfigurator: NSViewRepresentable {
    @Binding var isFullScreen: Bool

    func makeNSView(context: Context) -> ConfigView {
        let view = ConfigView()
        view.onFullScreenChange = { fullScreen in
            DispatchQueue.main.async { isFullScreen = fullScreen }
        }
        return view
    }

    func updateNSView(_ nsView: ConfigView, context: Context) {
        guard let window = nsView.window else { return }
        window.titlebarSeparatorStyle = .none
        DispatchQueue.main.async {
            Self.repositionTrafficLights(in: window)
        }
    }

    static func repositionTrafficLights(in window: NSWindow) {
        guard let contentView = window.contentView else { return }
        let windowHeight = contentView.frame.height
        guard windowHeight > 0 else { return }

        let buttonTypes: [NSWindow.ButtonType] = [.closeButton, .miniaturizeButton, .zoomButton]
        // The sidebar toggle button center is 24pt from window top
        // (44pt title bar, 4pt top padding, 30pt button → centered at 5+4+15=24).
        let targetCenterY = windowHeight - 24
        // Traffic light left margin and spacing (standard spacing is 20pt)
        let leftMargin: CGFloat = 17
        let buttonSpacing: CGFloat = 20

        for (index, type) in buttonTypes.enumerated() {
            guard let button = window.standardWindowButton(type) else { continue }
            guard let container = button.superview else { continue }
            let centerInContainer = container.convert(NSPoint(x: 0, y: targetCenterY), from: nil)
            var frame = button.frame
            frame.origin.y = centerInContainer.y - frame.height / 2
            frame.origin.x = leftMargin + CGFloat(index) * buttonSpacing
            button.frame = frame
        }
    }

    class ConfigView: NSView {
        var onFullScreenChange: ((Bool) -> Void)?

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            guard let window else { return }
            window.titlebarSeparatorStyle = .none
            window.isOpaque = false
            window.backgroundColor = .clear
            WindowConfigurator.repositionTrafficLights(in: window)

            NotificationCenter.default.addObserver(
                self, selector: #selector(didEnterFullScreen),
                name: NSWindow.willEnterFullScreenNotification, object: window
            )
            NotificationCenter.default.addObserver(
                self, selector: #selector(didExitFullScreen),
                name: NSWindow.willExitFullScreenNotification, object: window
            )
            NotificationCenter.default.addObserver(
                self, selector: #selector(windowDidResize),
                name: NSWindow.didResizeNotification, object: window
            )

            onFullScreenChange?(window.styleMask.contains(.fullScreen))
        }

        @objc private func didEnterFullScreen(_ n: Notification) {
            onFullScreenChange?(true)
        }

        @objc private func didExitFullScreen(_ n: Notification) {
            onFullScreenChange?(false)
        }

        @objc private func windowDidResize(_ n: Notification) {
            guard let window = n.object as? NSWindow else { return }
            WindowConfigurator.repositionTrafficLights(in: window)
        }

        deinit {
            NotificationCenter.default.removeObserver(self)
        }
    }
}

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
