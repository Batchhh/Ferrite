import SwiftUI

// MARK: - Code Preview View

struct CodePreviewView: View {
    @Environment(DecompilerService.self) var service
    @Environment(ProjectService.self) private var projectService
    @State var expandedProperties: Set<UInt32> = []
    @State var expandedMethods: Set<UInt32> = []
    @State var allExpanded = false
    @State var isSearching = false
    @State var searchQuery = ""
    @State var matchRanges: [NSRange] = []
    @State var currentMatchIndex = 0

    private var currentType: TypeInfo? {
        switch service.selection {
        case .type(let assemblyId, let token):
            return service.findType(assemblyId: assemblyId, token: token)
        case .member(let assemblyId, let typeToken, _):
            return service.findType(assemblyId: assemblyId, token: typeToken)
        default:
            return nil
        }
    }

    private func dismissSearch() {
        withAnimation(.easeInOut(duration: 0.2)) {
            isSearching = false
        }
        searchQuery = ""
        matchRanges = []
        currentMatchIndex = 0
    }

    private func toggleAllCollapsibles() {
        guard let type_ = currentType else { return }
        if allExpanded {
            expandedProperties.removeAll()
            expandedMethods.removeAll()
            allExpanded = false
        } else {
            expandedProperties = Set(type_.properties.map(\.token))
            expandedMethods = Set(type_.members.filter { $0.kind == .method }.map(\.token))
            allExpanded = true
        }
        service.codeAllExpanded = allExpanded
    }

    var body: some View {
        Group {
            if service.selection == nil {
                VStack(spacing: 14) {
                    Image(systemName: "chevron.left.forwardslash.chevron.right")
                        .font(.system(size: 36, weight: .ultraLight))
                        .foregroundStyle(.tertiary)
                    Text("Select a type or member\nto view its code.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.center)
                        .lineSpacing(3)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if !hasCode {
                VStack(spacing: 14) {
                    Image(systemName: "curlybraces")
                        .font(.system(size: 36, weight: .ultraLight))
                        .foregroundStyle(.tertiary)
                    Text("No code view for this selection.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ZStack(alignment: .bottomTrailing) {
                    VStack(spacing: 0) {
                        Rectangle()
                            .fill(Color(NSColor.separatorColor))
                            .frame(height: 1)
                        codeView
                            .clipped()
                            .id(service.selection)
                            .transition(.opacity)
                    }

                    if isSearching {
                        CodeSearchBar(
                            query: $searchQuery,
                            currentIndex: $currentMatchIndex,
                            totalMatches: matchRanges.count,
                            onDismiss: dismissSearch
                        )
                        .frame(maxHeight: .infinity, alignment: .topTrailing)
                        .padding(.top, 8)
                        .padding(.trailing, 16)
                        .transition(.opacity)
                    }

                }
                .clipped()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .animation(.easeInOut(duration: 0.25), value: service.selection)
        .background {
            Group {
                Button("") { toggleAllCollapsibles() }
                    .keyboardShortcut("r", modifiers: .command)
                Button("") {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        if isSearching {
                            dismissSearch()
                        } else {
                            isSearching = true
                        }
                    }
                }
                    .keyboardShortcut("f", modifiers: .command)
            }
            .hidden()
        }
        .onChange(of: service.codeCollapseToggleId) {
            toggleAllCollapsibles()
        }
        .onChange(of: service.selection) {
            expandedProperties.removeAll()
            expandedMethods.removeAll()
            allExpanded = false
            service.codeAllExpanded = false
            dismissSearch()
        }
        .onChange(of: searchQuery) {
            currentMatchIndex = 0
        }
    }

}
