import SwiftUI

struct CodeSearchBar: View {
    @Binding var query: String
    @Binding var currentIndex: Int
    let totalMatches: Int
    let onDismiss: () -> Void

    @FocusState private var isFocused: Bool

    private var matchLabel: String {
        if query.isEmpty { return "" }
        if totalMatches == 0 { return "No results" }
        return "\(currentIndex + 1) of \(totalMatches)"
    }

    var body: some View {
        HStack(spacing: 4) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 11, weight: .medium))
                .foregroundStyle(.secondary)

            TextField("Find", text: $query)
                .textFieldStyle(.plain)
                .font(.system(size: 12))
                .focused($isFocused)
                .onSubmit {
                    if NSApp.currentEvent?.modifierFlags.contains(.shift) == true {
                        navigatePrevious()
                    } else {
                        navigateNext()
                    }
                }
                .onExitCommand { onDismiss() }

            if !query.isEmpty {
                Text(matchLabel)
                    .font(.system(size: 10, weight: .medium))
                    .foregroundStyle(.tertiary)
                    .monospacedDigit()
                    .fixedSize()

                Divider()
                    .frame(height: 14)

                Button(action: navigatePrevious) {
                    Image(systemName: "chevron.up")
                        .font(.system(size: 10, weight: .semibold))
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .disabled(totalMatches == 0)

                Button(action: navigateNext) {
                    Image(systemName: "chevron.down")
                        .font(.system(size: 10, weight: .semibold))
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .disabled(totalMatches == 0)
            }

            Button(action: onDismiss) {
                Image(systemName: "xmark")
                    .font(.system(size: 10, weight: .semibold))
            }
            .buttonStyle(.plain)
            .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .frame(width: 280)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .shadow(color: .black.opacity(0.25), radius: 8, y: 4)
        .onAppear { isFocused = true }
    }

    private func navigateNext() {
        guard totalMatches > 0 else { return }
        currentIndex = (currentIndex + 1) % totalMatches
    }

    private func navigatePrevious() {
        guard totalMatches > 0 else { return }
        currentIndex = (currentIndex - 1 + totalMatches) % totalMatches
    }
}
