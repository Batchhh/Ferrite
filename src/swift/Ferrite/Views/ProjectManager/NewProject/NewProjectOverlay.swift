import SwiftUI
import AppKit

// MARK: - New Project Overlay

struct NewProjectOverlay: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @State private var isVisible = false
    @State private var escMonitor: Any?

    var body: some View {
        ZStack {
            Color.black.opacity(isVisible ? 0.25 : 0)
                .ignoresSafeArea()
                .onTapGesture { dismiss() }
                .animation(.easeOut(duration: 0.2), value: isVisible)

            NewProjectSheet(onDismiss: { dismiss() }) { name, tags in
                let project = projectService.createProject(name: name, tags: tags)
                projectService.openProject(project, in: service)
                dismiss()
            }
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 14))
            .overlay(
                RoundedRectangle(cornerRadius: 14)
                    .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
            )
            .shadow(color: .black.opacity(0.45), radius: 40, y: 12)
            .scaleEffect(isVisible ? 1 : 0.95)
            .opacity(isVisible ? 1 : 0)
            .padding(.top, OverlayLayout.topPadding)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
            .animation(.spring(duration: 0.25, bounce: 0.15), value: isVisible)

        }
        .onAppear {
            isVisible = true
            escMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
                guard event.keyCode == 53 else { return event }
                dismiss()
                return nil
            }
        }
        .onDisappear {
            if let monitor = escMonitor {
                NSEvent.removeMonitor(monitor)
                escMonitor = nil
            }
        }
    }

    private func dismiss() {
        withAnimation(.easeIn(duration: 0.15)) {
            isVisible = false
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
            projectService.showingNewProject = false
        }
    }
}
