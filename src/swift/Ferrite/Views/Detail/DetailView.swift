import SwiftUI

// Requires macOS 26 (Tahoe) for Liquid Glass APIs.

// MARK: - Detail View

struct DetailView: View {
    @Environment(DecompilerService.self) private var service

    var body: some View {
        Group {
            if let error = service.error {
                ContentUnavailableView(
                    "Could Not Parse Assembly",
                    systemImage: "exclamationmark.triangle",
                    description: Text(error)
                )
            } else if let selection = service.selection {
                selectionDetail(selection)
            } else {
                ContentUnavailableView {
                    Label("No Selection", systemImage: "sidebar.right")
                } description: {
                    Text("Select an item from the sidebar to inspect it.")
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    @ViewBuilder
    private func selectionDetail(_ selection: Selection) -> some View {
        switch selection {
        case .assembly(let id):
            if let entry = service.findAssembly(id: id) {
                AssemblyDetailView(summary: entry)
            }
        case .namespace(let assemblyId, let name):
            if let entry = service.findAssembly(id: assemblyId) {
                let types = service.getNamespaceTypes(assemblyId: assemblyId, namespace: name)
                NamespaceDetailView(namespaceName: name, types: types, assemblyName: entry.name)
            }
        case .type(let assemblyId, let token):
            if let type_ = service.findType(assemblyId: assemblyId, token: token) {
                TypeDetailView(type_: type_, assemblyId: assemblyId)
            }
        case .member(let assemblyId, let typeToken, let memberToken):
            if let type_ = service.findType(assemblyId: assemblyId, token: typeToken) {
                if let member = service.findMember(
                    assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken
                ) {
                    MemberDetailView(member: member, declaringType: type_)
                } else if let prop = service.findProperty(
                    assemblyId: assemblyId, typeToken: typeToken, propertyToken: memberToken
                ) {
                    PropertyDetailView(property: prop, declaringType: type_)
                } else if let event = service.findEvent(
                    assemblyId: assemblyId, typeToken: typeToken, eventToken: memberToken
                ) {
                    EventDetailView(event: event, declaringType: type_)
                }
            }
        }
    }
}

// MARK: - Assembly Detail

struct AssemblyDetailView: View {
    let summary: AssemblySummary

    private var totalTypes: Int {
        summary.namespaces.reduce(0) { $0 + Int($1.typeCount) }
    }

    private var subtitle: String {
        [summary.version, summary.targetFramework]
            .filter { !$0.isEmpty }
            .joined(separator: "  ·  ")
    }

    var body: some View {
        VStack(spacing: 0) {
            Spacer()

            VStack(spacing: 20) {
                VStack(spacing: 12) {
                    Image(systemName: "shippingbox")
                        .font(.system(size: 32, weight: .thin))
                        .foregroundStyle(.tertiary)
                        .padding(.bottom, 2)

                    Text(summary.name)
                        .font(.system(size: 20, weight: .semibold))
                        .textSelection(.enabled)

                    if !subtitle.isEmpty {
                        Text(subtitle)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundStyle(.secondary)
                    }
                }

                HStack(spacing: 14) {
                    IconStat(icon: "folder", value: summary.namespaces.count, label: "namespaces")
                    IconStat(icon: "cube", value: totalTypes, label: "types")
                }

                if !summary.assemblyReferences.isEmpty {
                    ReferencesDisclosure(references: summary.assemblyReferences)
                }
            }

            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Section Header

struct SectionHeader: View {
    let title: String
    var count: Int? = nil

    var body: some View {
        HStack {
            Group {
                if let count {
                    Text("\(title) (\(count))")
                } else {
                    Text(title)
                }
            }
            .font(.caption.weight(.semibold))
            .foregroundStyle(.secondary)
            .textCase(.uppercase)
            .tracking(0.5)
            Spacer()
        }
        .padding(.top, 12)
        .padding(.bottom, 4)
    }
}

// MARK: - Modifier Row

struct ModifierRow: View {
    let name: String

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "checkmark.circle.fill")
                .font(.caption)
                .foregroundStyle(.green)
                .frame(width: 16)
            Text(name)
                .font(.callout)
            Spacer()
        }
        .padding(.vertical, 5)
    }
}

