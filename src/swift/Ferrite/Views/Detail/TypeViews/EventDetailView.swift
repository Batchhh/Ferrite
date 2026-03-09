import SwiftUI

struct EventDetailView: View {
    let event: EventInfo
    let declaringType: TypeInfo

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                eventHeader
                    .padding(16)

                Divider().padding(.horizontal, 16)
                SectionHeader(title: "Type")
                    .padding(.horizontal, 16)
                HStack(spacing: 8) {
                    Text(event.eventType.isEmpty ? "object" : event.eventType)
                        .font(.system(.callout, design: .monospaced))
                        .foregroundStyle(.primary)
                    Spacer()
                }
                .padding(.vertical, 4)
                .padding(.horizontal, 16)

                Divider().padding(.horizontal, 16)
                SectionHeader(title: "Accessors")
                    .padding(.horizontal, 16)
                if event.addToken != nil {
                    ModifierRow(name: "add").padding(.horizontal, 16)
                }
                if event.removeToken != nil {
                    ModifierRow(name: "remove").padding(.horizontal, 16)
                }
                if event.raiseToken != nil {
                    ModifierRow(name: "raise").padding(.horizontal, 16)
                }

                Divider().padding(.horizontal, 16)
                SectionHeader(title: "Declaring Type")
                    .padding(.horizontal, 16)
                HStack(spacing: 8) {
                    Image(systemName: iconForTypeKind(declaringType.kind))
                        .foregroundStyle(colorForTypeKind(declaringType.kind))
                        .frame(width: 16)
                    Text(declaringType.fullName)
                        .font(.system(.callout, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    Spacer()
                }
                .padding(.vertical, 6)
                .padding(.horizontal, 16)
                .padding(.bottom, 16)
            }
        }
        .scrollIndicators(.hidden)
    }

    @ViewBuilder
    private var eventHeader: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: iconForMemberKind(.event))
                    .font(.title2)
                    .foregroundStyle(colorForMemberKind(.event))
                    .frame(width: 28, alignment: .center)
                VStack(alignment: .leading, spacing: 3) {
                    Text(event.name)
                        .font(.title3.weight(.semibold))
                        .textSelection(.enabled)
                    Text("\(declaringType.fullName).\(event.name)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                        .lineLimit(3)
                }
                Spacer(minLength: 4)
            }
            Divider()
            HStack(spacing: 6) {
                KindPill(text: "event", color: colorForMemberKind(.event))
                Spacer()
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .modifier(GlassEffectModifier(shape: .rect(cornerRadius: 12)))
    }
}
