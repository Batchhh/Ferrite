import SwiftUI
import AppKit

// MARK: - Welcome Project Row

struct WelcomeProjectRow: View {
    let project: Project
    let onOpen: () -> Void
    let onDelete: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: onOpen) {
            HStack(spacing: 8) {
                Image(systemName: "doc.text")
                    .font(.system(size: 10))
                    .foregroundColor(isHovered ? .secondary : Color.white.opacity(0.2))
                    .frame(width: 16, alignment: .center)

                Text(project.name)
                    .font(.system(size: 13))
                    .foregroundStyle(isHovered ? .primary : .tertiary)
                    .lineLimit(1)
                    .truncationMode(.tail)

                Spacer(minLength: 12)

                Text(relativeDateText)
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(.white.opacity(0.15))
            }
            .padding(.vertical, 6)
            .padding(.horizontal, 10)
            .contentShape(Rectangle())
            .background(
                RoundedRectangle(cornerRadius: 7, style: .continuous)
                    .fill(.white.opacity(isHovered ? 0.06 : 0))
            )
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
            if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
        }
        .animation(.easeInOut(duration: 0.12), value: isHovered)
        .contextMenu {
            Button("Delete Project", role: .destructive, action: onDelete)
        }
    }

    private var relativeDateText: String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: project.lastOpenedAt, relativeTo: Date())
    }
}

// MARK: - Drop Zone Card

struct DropZoneCard: View {
    @State private var iconOffset: CGFloat = 0

    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "arrow.down.circle.fill")
                .font(.system(size: 30, weight: .regular))
                .foregroundStyle(Color(NSColor.controlAccentColor))
                .offset(y: iconOffset)
                .animation(
                    .easeInOut(duration: 1.2).repeatForever(autoreverses: true),
                    value: iconOffset
                )

            Text("Drop DLL or EXE to Open")
                .font(.system(size: 17, weight: .semibold))
                .foregroundStyle(.primary)

            Text("Ferrite will load the assembly immediately.")
                .font(.system(size: 13))
                .foregroundStyle(.secondary)
        }
        .multilineTextAlignment(.center)
        .padding(.horizontal, 30)
        .padding(.vertical, 24)
        .background(
            RoundedRectangle(cornerRadius: 26, style: .continuous)
                .fill(.black.opacity(0.42))
        )
        .overlay {
            RoundedRectangle(cornerRadius: 26, style: .continuous)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        }
        .shadow(color: .black.opacity(0.25), radius: 24, y: 10)
        .onAppear { iconOffset = -4 }
    }
}

// MARK: - Keyboard Shortcut Hint

func shortcutHint(_ text: String) -> some View {
    Text(text)
        .font(.system(size: 11, weight: .medium, design: .rounded))
        .foregroundStyle(.quaternary)
        .frame(height: 22)
        .padding(.horizontal, 6)
        .background(
            RoundedRectangle(cornerRadius: 5)
                .fill(.white.opacity(0.06))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 5)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        )
}
