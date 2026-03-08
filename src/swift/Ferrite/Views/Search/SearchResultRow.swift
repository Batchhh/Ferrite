import SwiftUI

// MARK: - Popover Filter Chip

struct PopoverFilterChip: View {
    let filter: SearchFilter
    let isActive: Bool
    let action: () -> Void
    @State private var isHovered = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: filter.icon)
                    .font(.system(size: 9))
                    .foregroundStyle(isActive ? .primary : .tertiary)
                    .frame(width: 10, alignment: .center)

                Text(filter.label)
                    .font(.system(size: 10, weight: isActive ? .semibold : .regular))
                    .foregroundStyle(isActive ? .primary : .secondary)
                    .lineLimit(1)
            }
            .padding(.horizontal, 6)
            .padding(.vertical, 5)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                RoundedRectangle(cornerRadius: 5)
                    .fill(.white.opacity(isActive ? 0.1 : isHovered ? 0.04 : 0))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 5)
                    .strokeBorder(.white.opacity(isActive ? 0.12 : 0), lineWidth: 0.5)
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { isHovered = $0 }
        .animation(.easeOut(duration: 0.1), value: isHovered)
        .animation(.easeOut(duration: 0.1), value: isActive)
    }
}

// MARK: - Search Result Row

struct SearchResultRow: View {
    let item: SearchItem
    let isSelected: Bool
    @State private var isHovered = false

    private var isHighlighted: Bool { isSelected || isHovered }

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: item.icon)
                .font(.system(size: 11))
                .foregroundStyle(.tertiary)
                .frame(width: 16, alignment: .center)

            VStack(alignment: .leading, spacing: 1) {
                Text(item.title)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(.primary)
                    .lineLimit(1)
                Text(item.subtitle)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer(minLength: 4)

            Text(item.assemblyName)
                .font(.system(size: 10))
                .foregroundStyle(.quaternary)
                .lineLimit(1)

            if isSelected {
                Text("\u{21A9}")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundStyle(.quaternary)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(.white.opacity(isHighlighted ? 0.06 : 0))
        )
        .contentShape(Rectangle())
        .onHover { isHovered = $0 }
        .animation(.easeOut(duration: 0.1), value: isSelected)
        .animation(.easeOut(duration: 0.1), value: isHovered)
    }
}
