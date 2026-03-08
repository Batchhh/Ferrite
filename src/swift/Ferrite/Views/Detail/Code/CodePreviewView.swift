import SwiftUI

// MARK: - Code Preview View

struct CodePreviewView: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @State private var expandedProperties: Set<UInt32> = []
    @State private var expandedMethods: Set<UInt32> = []
    @State private var allExpanded = false
    @State private var isSearching = false
    @State private var searchQuery = ""
    @State private var matchRanges: [NSRange] = []
    @State private var currentMatchIndex = 0

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

    private var lines: [CodeLine] {
        switch service.selection {
        case .type(let assemblyId, let token):
            if let type_ = service.findType(assemblyId: assemblyId, token: token) {
                if let decompiled = service.decompileType(assemblyId: assemblyId, token: token) {
                    return parseDecompiledCode(decompiled, type_: type_, assemblyId: assemblyId, expandedProperties: expandedProperties, expandedMethods: expandedMethods)
                }
                return generateTypeCode(type_, expandedProperties: expandedProperties, expandedMethods: expandedMethods)
            }
        case .member(let assemblyId, let typeToken, let memberToken):
            if let type_ = service.findType(assemblyId: assemblyId, token: typeToken),
               let member = service.findMember(
                   assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken
               ) {
                if member.kind == .method,
                   let decompiled = service.decompileType(assemblyId: assemblyId, token: typeToken),
                   let extracted = extractMethodFromDecompiled(decompiled, member: member, typeName: type_.name) {
                    // Expand all methods — the extracted code contains only this one method,
                    // but the token queue may assign a different overload's token by name
                    let allMethodTokens = Set(type_.members.filter { $0.kind == .method }.map(\.token))
                    return parseDecompiledCode(extracted, type_: type_, assemblyId: assemblyId, expandedMethods: allMethodTokens)
                }
                return generateMemberCode(member, declaringType: type_)
            }
        default:
            break
        }
        return []
    }

    private var hasCode: Bool {
        switch service.selection {
        case .type, .member: return true
        default:             return false
        }
    }

    private var codeView: some View {
        CodeView(
            lines: lines,
            searchQuery: isSearching ? searchQuery : "",
            currentMatchIndex: currentMatchIndex,
            matchRanges: $matchRanges,
            resolveType: { name in
                let currentAssemblyId: String? = switch service.selection {
                case .type(let aid, _): aid
                case .member(let aid, _, _): aid
                default: nil
                }
                return service.findTypeByShortName(name, preferredAssemblyId: currentAssemblyId)
            },
            onNavigate: { assemblyId, token in
                service.selection = .type(assemblyId: assemblyId, token: token)
            },
            onNavigateMember: { assemblyId, typeToken, memberToken in
                service.selection = .member(assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken)
            },
            onToggleProperty: { propToken in
                if expandedProperties.contains(propToken) {
                    expandedProperties.remove(propToken)
                } else {
                    expandedProperties.insert(propToken)
                }
            },
            onToggleMethod: { methodToken in
                if expandedMethods.contains(methodToken) {
                    expandedMethods.remove(methodToken)
                } else {
                    expandedMethods.insert(methodToken)
                }
            }
        )
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
