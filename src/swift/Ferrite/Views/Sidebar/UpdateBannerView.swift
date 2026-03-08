import SwiftUI
import AppKit

struct UpdateBannerView: View {
    @Environment(UpdateService.self) private var updateService
    @State private var isHovered = false

    var body: some View {
        if updateService.isUpdateAvailable && !updateService.isDismissed {
            banner
                .transition(.opacity.combined(with: .move(edge: .top)))
                .animation(.easeInOut(duration: 0.2), value: updateService.isDismissed)
        }
    }

    private var banner: some View {
        Button {
            if let url = updateService.releaseURL {
                NSWorkspace.shared.open(url)
            }
        } label: {
            HStack(spacing: 8) {
                Image(systemName: "arrow.up.circle.fill")
                    .font(.system(size: 11))
                    .foregroundStyle(Color(NSColor.controlAccentColor))

                VStack(alignment: .leading, spacing: 1) {
                    Text("Update available")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundStyle(.primary)

                    if let version = updateService.latestVersion {
                        Text("v\(version)")
                            .font(.system(size: 10, weight: .medium, design: .monospaced))
                            .foregroundStyle(.secondary)
                    }
                }

                Spacer(minLength: 0)

                Button {
                    updateService.dismiss()
                } label: {
                    Image(systemName: "xmark")
                        .font(.system(size: 9, weight: .medium))
                        .foregroundStyle(.white.opacity(0.3))
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 6)
                    .fill(.white.opacity(isHovered ? 0.07 : 0.04))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 6)
                    .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
            )
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 8)
        .padding(.top, 4)
        .onHover { hovering in
            isHovered = hovering
            if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
        }
        .animation(.easeInOut(duration: 0.12), value: isHovered)
    }
}
