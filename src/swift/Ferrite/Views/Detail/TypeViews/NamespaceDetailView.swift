import SwiftUI

// MARK: - Namespace Detail

struct NamespaceDetailView: View {
    let namespaceName: String
    let types: [TypeSummary]
    let assemblyName: String

    private var displayName: String {
        namespaceName.isEmpty ? "(global)" : namespaceName
    }

    private var kindCounts: [(kind: TypeKind, count: Int)] {
        var map: [TypeKind: Int] = [:]
        for type_ in types {
            map[type_.kind, default: 0] += 1
        }
        // Stable order: class, struct, interface, enum, delegate
        let order: [TypeKind] = [.`class`, .`struct`, .interface, .`enum`, .delegate]
        return order.compactMap { kind in
            guard let count = map[kind] else { return nil }
            return (kind, count)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            Spacer()

            VStack(spacing: 28) {
                VStack(spacing: 12) {
                    Image(systemName: "folder")
                        .font(.system(size: 32, weight: .thin))
                        .foregroundStyle(.tertiary)
                        .padding(.bottom, 2)

                    Text(displayName)
                        .font(.system(size: 20, weight: .semibold))
                        .textSelection(.enabled)

                    HStack(spacing: 4) {
                        Image(systemName: "shippingbox")
                            .font(.system(size: 9))
                        Text(assemblyName)
                    }
                    .font(.system(.caption2, design: .monospaced))
                    .foregroundStyle(.quaternary)
                }

                if !kindCounts.isEmpty {
                    HStack(spacing: 14) {
                        ForEach(kindCounts, id: \.kind) { item in
                            HStack(spacing: 5) {
                                Image(systemName: iconForTypeKind(item.kind))
                                    .font(.system(size: 10))
                                    .foregroundStyle(.tertiary)
                                Text("\(item.count) \(typeKindLabel(item.kind))\(item.count == 1 ? "" : "s")")
                                    .font(.caption2)
                                    .foregroundStyle(.quaternary)
                            }
                        }
                    }
                }
            }

            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
