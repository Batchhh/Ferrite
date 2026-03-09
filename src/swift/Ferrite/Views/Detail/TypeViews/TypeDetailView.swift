import SwiftUI

// MARK: - Type Detail

struct TypeDetailView: View {
    let type_: TypeInfo
    let assemblyId: String

    var body: some View {
        ScrollView {
            typeHeader
                .padding(16)
        }
        .scrollIndicators(.hidden)
    }

    @ViewBuilder
    private var typeHeader: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: iconForTypeKind(type_.kind))
                    .font(.title2)
                    .foregroundStyle(colorForTypeKind(type_.kind))
                    .frame(width: 28, alignment: .center)
                VStack(alignment: .leading, spacing: 3) {
                    Text(type_.name)
                        .font(.title3.weight(.semibold))
                    Text(type_.fullName)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                        .lineLimit(3)
                }
                Spacer(minLength: 4)
                AccessPill(text: visibilityLabel(type_.attributes.visibility))
            }
            Divider()
            HStack(spacing: 6) {
                KindPill(text: typeKindLabel(type_.kind), color: colorForTypeKind(type_.kind))
                if type_.attributes.isStatic {
                    ModifierPill(text: "static")
                }
                if type_.attributes.isAbstract && !type_.attributes.isStatic {
                    ModifierPill(text: "abstract")
                }
                if type_.attributes.isSealed && !type_.attributes.isStatic {
                    ModifierPill(text: "sealed")
                }
                Spacer()
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .modifier(GlassEffectModifier(shape: .rect(cornerRadius: 12)))
    }
}
