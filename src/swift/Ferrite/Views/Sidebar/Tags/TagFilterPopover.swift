import SwiftUI

/// Popover for filtering sidebar items by tag.
struct TagFilterPopover: View {
    @Environment(ItemTagService.self) private var tagService

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Filter by Tag")
                .font(.caption2.weight(.semibold))
                .foregroundStyle(.tertiary)
                .textCase(.uppercase)
                .tracking(0.6)
                .padding(.horizontal, 14)
                .padding(.top, 12)
                .padding(.bottom, 8)

            Divider()

            VStack(spacing: 0) {
                ForEach(ItemTag.allCases, id: \.self) { tag in
                    Button {
                        tagService.toggleFilter(tag)
                    } label: {
                        HStack(spacing: 10) {
                            Circle()
                                .fill(tag.color)
                                .frame(width: 8, height: 8)
                            Text(tag.displayName)
                                .font(.callout)
                                .foregroundStyle(.primary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                            if tagService.activeFilters.contains(tag) {
                                Image(systemName: "checkmark")
                                    .font(.system(size: 10, weight: .semibold))
                                    .foregroundStyle(.secondary)
                            }
                        }
                        .padding(.horizontal, 14)
                        .padding(.vertical, 9)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                }

                if tagService.isFiltering {
                    Divider()
                        .padding(.horizontal, 14)

                    Button {
                        tagService.clearFilters()
                    } label: {
                        HStack(spacing: 10) {
                            Image(systemName: "xmark.circle")
                                .font(.system(size: 11))
                                .foregroundStyle(.secondary)
                                .frame(width: 8, alignment: .center)
                            Text("Clear All")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .padding(.horizontal, 14)
                        .padding(.vertical, 9)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.bottom, 6)
        }
        .frame(width: 200)
    }
}
