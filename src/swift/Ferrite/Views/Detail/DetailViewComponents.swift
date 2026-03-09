import SwiftUI

// MARK: - Stat Components

struct IconStat: View {
    let icon: String
    let value: Int
    let label: String

    var body: some View {
        HStack(spacing: 5) {
            Image(systemName: icon)
                .font(.system(size: 10))
                .foregroundStyle(.tertiary)
            Text("\(value) \(label)")
                .font(.caption2)
                .foregroundStyle(.quaternary)
        }
    }
}

struct StatItem: View {
    let value: String
    let label: String

    var body: some View {
        VStack(spacing: 2) {
            Text(value)
                .font(.system(size: 18, weight: .medium, design: .rounded).monospacedDigit())
                .foregroundStyle(.primary)
            Text(label)
                .font(.caption2)
                .foregroundStyle(.quaternary)
                .textCase(.uppercase)
                .tracking(0.3)
        }
        .frame(minWidth: 64)
    }
}

// MARK: - References Disclosure

struct ReferencesDisclosure: View {
    let references: [String]
    @State private var isExpanded = false

    var body: some View {
        VStack(spacing: 0) {
            Button {
                withAnimation(.easeInOut(duration: 0.2)) { isExpanded.toggle() }
            } label: {
                HStack(spacing: 4) {
                    Image(systemName: "chevron.right")
                        .font(.system(size: 8, weight: .bold))
                        .foregroundStyle(.quaternary)
                        .rotationEffect(.degrees(isExpanded ? 90 : 0))
                    Text("\(references.count) references")
                        .font(.caption)
                        .foregroundStyle(.quaternary)
                }
            }
            .buttonStyle(.plain)

            if isExpanded {
                ScrollView {
                    VStack(spacing: 2) {
                        ForEach(references, id: \.self) { ref in
                            Text(ref)
                                .font(.system(.caption2, design: .monospaced))
                                .foregroundStyle(.tertiary)
                                .lineLimit(1)
                        }
                    }
                    .padding(.top, 8)
                }
                .frame(maxHeight: 160)
                .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
    }
}

// MARK: - Pill Components

struct AccessPill: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.caption2.weight(.medium))
            .foregroundStyle(Color(NSColor.controlAccentColor))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(Color(NSColor.controlAccentColor).opacity(0.12), in: .capsule)
    }
}

struct KindPill: View {
    let text: String
    let color: Color

    var body: some View {
        Text(text)
            .font(.caption2.weight(.medium))
            .foregroundStyle(color)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(color.opacity(0.12), in: .capsule)
    }
}

struct ModifierPill: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.caption2)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(.secondary.opacity(0.1), in: .capsule)
    }
}
