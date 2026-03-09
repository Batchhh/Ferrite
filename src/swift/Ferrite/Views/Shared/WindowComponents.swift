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

