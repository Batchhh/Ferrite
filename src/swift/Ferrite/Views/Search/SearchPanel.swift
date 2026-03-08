import SwiftUI

struct SearchPanel: View {
    @Environment(SearchService.self) private var searchService
    @Environment(DecompilerService.self) private var service
    @FocusState private var isFieldFocused: Bool
    @State private var selectedIndex: Int = 0
    @State private var isVisible = false
    @State private var showFilters = false

    private var hasActiveFilters: Bool { !searchService.activeFilters.isEmpty }
    private var activeCount: Int { searchService.activeFilters.count }
    private var showingRecents: Bool { searchService.query.isEmpty && !searchService.recentItems.isEmpty }
    private var displayedItems: [SearchItem] { showingRecents ? searchService.recentItems : searchService.results }

    var body: some View {
        ZStack {
            Color.black.opacity(isVisible ? 0.25 : 0)
                .ignoresSafeArea()
                .onTapGesture {
                    if showFilters {
                        withAnimation(.spring(duration: 0.25, bounce: 0.1)) {
                            showFilters = false
                        }
                    } else {
                        dismiss()
                    }
                }
                .animation(.easeOut(duration: 0.2), value: isVisible)

            searchContainer
                .overlay(alignment: .topTrailing) {
                    if showFilters {
                        FilterPopover(selectedIndex: $selectedIndex)
                            // Offset derived from: x = -horizontal padding (18), y = search field height (~52)
                            .offset(x: -18, y: 52)
                            .transition(.opacity.combined(with: .scale(scale: 0.95, anchor: .topTrailing)))
                    }
                }
                .scaleEffect(isVisible ? 1 : 0.95)
                .opacity(isVisible ? 1 : 0)
                .padding(.top, OverlayLayout.topPadding)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
                .animation(.spring(duration: 0.25, bounce: 0.15), value: isVisible)
        }
        .onAppear {
            selectedIndex = 0
            isVisible = true
            focusSearchField()
        }
    }

    // MARK: - Search Container

    private var searchContainer: some View {
        VStack(spacing: 0) {
            searchField

            if hasActiveFilters {
                ActiveFilterChips(selectedIndex: $selectedIndex)
            }

            if showingRecents || !searchService.results.isEmpty || !searchService.query.isEmpty {
                Divider().opacity(0.5)
                resultsList
            }
        }
        .frame(width: 520)
        .fixedSize(horizontal: false, vertical: true)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 14))
        .overlay(
            RoundedRectangle(cornerRadius: 14)
                .strokeBorder(.white.opacity(0.08), lineWidth: 0.5)
        )
        .shadow(color: .black.opacity(0.45), radius: 40, y: 12)
    }

    // MARK: - Search Field

    private var searchField: some View {
        HStack(spacing: 12) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 18, weight: .light))
                .foregroundStyle(.secondary)

            TextField("Search anything\u{2026}", text: Binding(
                get: { searchService.query },
                set: {
                    searchService.query = $0
                    searchService.debouncedSearch()
                    selectedIndex = 0
                }
            ))
            .textFieldStyle(.plain)
            .font(.system(size: 18, weight: .light))
            .focused($isFieldFocused)
            .onKeyPress(.return) {
                selectCurrent()
                return .handled
            }
            .onKeyPress(.upArrow) {
                withAnimation(.easeOut(duration: 0.1)) {
                    if selectedIndex > 0 { selectedIndex -= 1 }
                }
                return .handled
            }
            .onKeyPress(.downArrow) {
                withAnimation(.easeOut(duration: 0.1)) {
                    if selectedIndex < displayedItems.count - 1 { selectedIndex += 1 }
                }
                return .handled
            }
            .onKeyPress(.escape) {
                if showFilters {
                    withAnimation(.spring(duration: 0.25, bounce: 0.1)) {
                        showFilters = false
                    }
                    return .handled
                }
                dismiss()
                return .handled
            }

            if !searchService.query.isEmpty {
                Button {
                    withAnimation(.easeOut(duration: 0.15)) {
                        searchService.query = ""
                        searchService.performSearch()
                    }
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 14))
                        .foregroundStyle(.quaternary)
                }
                .buttonStyle(.plain)
            }

            FilterToggleButton(
                hasActiveFilters: hasActiveFilters,
                activeCount: activeCount,
                isShowingFilters: showFilters
            ) {
                withAnimation(.spring(duration: 0.25, bounce: 0.1)) {
                    showFilters.toggle()
                }
            }

            if searchService.query.isEmpty {
                KeyboardHint(text: "\u{2318}K")
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 14)
    }

    // MARK: - Results

    private var resultsList: some View {
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

    private func dismiss() {
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

    private func focusSearchField() {
        DispatchQueue.main.async {
            isFieldFocused = true
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            isFieldFocused = true
        }
    }

    private func selectCurrent() {
        guard !displayedItems.isEmpty,
              selectedIndex < displayedItems.count else { return }
        select(displayedItems[selectedIndex])
    }

    private func select(_ item: SearchItem) {
        searchService.addRecent(item)
        service.selection = item.selection
        dismiss()
    }
}
