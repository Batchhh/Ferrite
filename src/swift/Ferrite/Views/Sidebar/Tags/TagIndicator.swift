import SwiftUI

/// Compact colored dots showing which tags are applied to a sidebar item.
struct TagIndicator: View {
    let tags: Set<ItemTag>

    var body: some View {
        if !tags.isEmpty {
            HStack(spacing: 2) {
                ForEach(ItemTag.allCases.filter { tags.contains($0) }, id: \.self) { tag in
                    Circle()
                        .fill(tag.color)
                        .frame(width: 6, height: 6)
                }
            }
        }
    }
}
