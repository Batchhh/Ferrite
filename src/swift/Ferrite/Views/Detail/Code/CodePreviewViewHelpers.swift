import SwiftUI

// MARK: - Code Preview View Helpers

extension CodePreviewView {
    var lines: [CodeLine] {
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

    var hasCode: Bool {
        switch service.selection {
        case .type, .member: return true
        default:             return false
        }
    }

    var codeView: some View {
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
}
