import SwiftUI
import AppKit

// MARK: - Count Badge

/// Plain count displayed at trailing edge of sidebar rows — no capsule background.
struct CountBadge: View {
    let count: Int

    var body: some View {
        Text("\(count)")
            .font(.system(size: 11).monospacedDigit())
            .foregroundStyle(.quaternary)
    }
}

// MARK: - Sidebar Interactive Row

struct SidebarInteractiveRowModifier: ViewModifier {
    @Binding var isHovered: Bool
    var isSelected: Bool

    func body(content: Content) -> some View {
        content
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 12)
            .padding(.vertical, 4)
            .contentShape(Rectangle())
            .padding(.horizontal, 8)
            .background(
                RoundedRectangle(cornerRadius: 6)
                    .fill(.primary.opacity(isSelected ? 0.10 : (isHovered ? 0.07 : 0)))
                    .padding(.horizontal, 8)
            )
            .onHover { hovering in
                isHovered = hovering
                if hovering {
                    NSCursor.pointingHand.set()
                } else {
                    NSCursor.arrow.set()
                }
            }
            .animation(.easeInOut(duration: 0.1), value: isHovered)
            .animation(.easeInOut(duration: 0.1), value: isSelected)
    }
}

extension View {
    func sidebarInteractiveRow(isHovered: Binding<Bool>, isSelected: Bool = false) -> some View {
        modifier(SidebarInteractiveRowModifier(isHovered: isHovered, isSelected: isSelected))
    }
}
