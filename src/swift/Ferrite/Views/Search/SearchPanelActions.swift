import SwiftUI

extension SearchPanel {
    // MARK: - Results

    var resultsList: some View {
        Group {
            if displayedItems.isEmpty && !searchService.query.isEmpty {
                HStack(spacing: 8) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 12))
                        .foregroundStyle(.quaternary)
                    Text("No results for \"\(searchService.query)\"")
                        .font(.system(size: 13))
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 20)
                .transition(.opacity)
            } else {
                ScrollViewReader { proxy in
                    ScrollView(.vertical, showsIndicators: false) {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(Array(displayedItems.enumerated()), id: \.element.id) { index, item in
                                SearchResultRow(item: item, isSelected: index == selectedIndex)
                                    .id(item.id)
                                    .onTapGesture { select(item) }
                            }
                        }
                        .padding(.vertical, 6)
                        .padding(.horizontal, 6)
                    }
                    .frame(maxHeight: 320)
                    .onChange(of: selectedIndex) { _, newValue in
                        guard newValue < displayedItems.count else { return }
                        withAnimation(.easeOut(duration: 0.1)) {
                            proxy.scrollTo(displayedItems[newValue].id, anchor: .center)
                        }
                    }
                }
            }
        }
    }

    // MARK: - Actions

    func dismiss() {
        withAnimation(.easeIn(duration: 0.15)) {
            isVisible = false
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
            searchService.isPresented = false
            searchService.query = ""
            searchService.performSearch()
            showFilters = false
        }
    }

    func focusSearchField() {
        DispatchQueue.main.async {
            isFieldFocused = true
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            isFieldFocused = true
        }
    }

    func selectCurrent() {
        guard !displayedItems.isEmpty,
              selectedIndex < displayedItems.count else { return }
        select(displayedItems[selectedIndex])
    }

    func select(_ item: SearchItem) {
        searchService.addRecent(item)
        service.selection = item.selection
        dismiss()
    }
}
