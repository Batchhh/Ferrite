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
            if let type_ = service.findType(assemblyId: assemblyId, token: typeToken),
               let member = service.findMember(
                   assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken
               ) {
                MemberDetailView(member: member, declaringType: type_)
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
