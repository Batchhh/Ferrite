import SwiftUI

private let typeFilters: [SearchFilter] = [.class_, .interface, .struct_, .enum_, .delegate]
private let memberFilters: [SearchFilter] = [.method, .field, .property, .event, .constant]

// MARK: - Filter Toggle Button

struct FilterToggleButton: View {
    let hasActiveFilters: Bool
    let activeCount: Int
    let isShowingFilters: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: "line.3.horizontal.decrease")
                    .font(.system(size: 10, weight: .medium))

                if hasActiveFilters {
                    Text("\(activeCount)")
                        .font(.system(size: 10, weight: .bold, design: .rounded))
                }
            }
            .foregroundStyle(hasActiveFilters || isShowingFilters ? .primary : .quaternary)
            .frame(height: 22)
            .padding(.horizontal, 6)
            .background(
                RoundedRectangle(cornerRadius: 5)
                    .fill(.white.opacity(hasActiveFilters || isShowingFilters ? 0.08 : 0.06))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 5)
                    .strokeBorder(.white.opacity(hasActiveFilters || isShowingFilters ? 0.12 : 0.08), lineWidth: 0.5)
            )
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Filter Popover

struct FilterPopover: View {
    @Environment(SearchService.self) private var searchService
    @Binding var selectedIndex: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            popoverSection("Types", filters: typeFilters)
            Divider().opacity(0.2).padding(.horizontal, 8)
            popoverSection("Members", filters: memberFilters)
        }
        .padding(10)
        .frame(width: 200)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        )
        .shadow(color: .black.opacity(0.3), radius: 20, y: 8)
    }

    private func popoverSection(_ title: String, filters: [SearchFilter]) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.system(size: 9, weight: .semibold))
                .foregroundStyle(.tertiary)
                .textCase(.uppercase)
                .tracking(0.5)
                .padding(.horizontal, 4)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 4) {
                ForEach(filters, id: \.self) { filter in
                    PopoverFilterChip(
                        filter: filter,
                        isActive: searchService.activeFilters.contains(filter)
                    ) {
                        withAnimation(.easeOut(duration: 0.15)) {
                            searchService.toggleFilter(filter)
                            selectedIndex = 0
                        }
                    }
                }
            }
        }
    }
}

// MARK: - Active Filter Chips

struct ActiveFilterChips: View {
    @Environment(SearchService.self) private var searchService
    @Binding var selectedIndex: Int

    private var activeCount: Int { searchService.activeFilters.count }

    var body: some View {
        HStack(spacing: 6) {
            ForEach(Array(searchService.activeFilters).sorted(by: { $0.label < $1.label }), id: \.self) { filter in
                Button {
                    withAnimation(.easeOut(duration: 0.15)) {
                        searchService.toggleFilter(filter)
                        selectedIndex = 0
                    }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: filter.icon)
                            .font(.system(size: 8))
                        Text(filter.label)
                            .font(.system(size: 10, weight: .medium))
                        Image(systemName: "xmark")
                            .font(.system(size: 7, weight: .bold))
                            .foregroundStyle(.tertiary)
                    }
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(
                        RoundedRectangle(cornerRadius: 6)
                            .fill(.white.opacity(0.06))
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 6)
                            .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
                    )
                }
                .buttonStyle(.plain)
                .transition(.opacity.combined(with: .scale(scale: 0.8)))
            }

            if activeCount >= 2 {
                Button {
                    withAnimation(.easeOut(duration: 0.15)) {
                        searchService.activeFilters.removeAll()
                        searchService.performSearch()
                        selectedIndex = 0
                    }
                } label: {
                    HStack(spacing: 2) {
                        Image(systemName: "xmark")
                            .font(.system(size: 7, weight: .bold))
                        Text("Clear all")
                            .font(.system(size: 10, weight: .medium))
                    }
                    .foregroundStyle(.tertiary)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .transition(.opacity.combined(with: .move(edge: .top)))
    }
}

// MARK: - Keyboard Hint

struct KeyboardHint: View {
    let text: String

    var body: some View {
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
}
