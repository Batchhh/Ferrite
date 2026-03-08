import SwiftUI
import AppKit

// MARK: - Project Manager Row

struct ProjectManagerRow: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    let project: Project
    let isCurrent: Bool
    let isSelected: Bool
    var onDelete: (() -> Void)?
    @State private var isHovered = false

    private var isHighlighted: Bool { isSelected || isHovered }

    private var projectTags: [ProjectTag] {
        projectService.tags(for: project)
    }

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: isCurrent ? "folder.fill" : "folder")
                .font(.system(size: 11))
                .foregroundStyle(isCurrent ? .primary : .tertiary)
                .frame(width: 16, alignment: .center)

            VStack(alignment: .leading, spacing: 1) {
                HStack(spacing: 6) {
                    Text(project.name)
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(.primary)
                        .lineLimit(1)

                    if !projectTags.isEmpty {
                        HStack(spacing: 3) {
                            ForEach(projectTags.prefix(3)) { tag in
                                Circle()
                                    .fill(tag.color.color)
                                    .frame(width: 5, height: 5)
                            }
                        }
                    }
                }
                Text(metaText)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer(minLength: 4)

            if isHovered {
                Button {
                    onDelete?()
                } label: {
                    Image(systemName: "xmark")
                        .font(.system(size: 9, weight: .semibold))
                        .foregroundStyle(.secondary)
                        .frame(width: 20, height: 20)
                        .background(
                            RoundedRectangle(cornerRadius: 5)
                                .fill(.white.opacity(0.06))
                        )
                }
                .buttonStyle(.plain)
                .transition(.opacity)
            } else if isSelected {
                Text("\u{21A9}")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundStyle(.quaternary)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(.white.opacity(isHighlighted ? 0.06 : 0))
        )
        .contentShape(Rectangle())
        .onHover { isHovered = $0 }
        .contextMenu {
            Button("Delete Project", role: .destructive) {
                onDelete?()
            }
        }
        .animation(.easeOut(duration: 0.1), value: isSelected)
        .animation(.easeOut(duration: 0.1), value: isHovered)
    }

    private var metaText: String {
        let count = project.dllPaths.count
        let assemblies = count == 1 ? "1 assembly" : "\(count) assemblies"
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        let date = formatter.localizedString(for: project.lastOpenedAt, relativeTo: Date())
        return "\(assemblies)  ·  \(date)"
    }
}
