import SwiftUI

// MARK: - Project Switcher Bar

struct ProjectSwitcherBar: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    let project: Project
    @State private var showPicker = false

    var body: some View {
        VStack(spacing: 0) {
            Divider()
            Button {
                showPicker = true
            } label: {
                HStack(spacing: 8) {
                    Image(systemName: "folder.fill")
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .frame(width: 16, alignment: .center)
                    Text(project.name)
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(.primary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Spacer()
                    Image(systemName: "chevron.up.chevron.down")
                        .font(.system(size: 9, weight: .medium))
                        .foregroundStyle(.quaternary)
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 11)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .popover(isPresented: $showPicker, arrowEdge: .bottom) {
                ProjectPickerPopover(isPresented: $showPicker)
            }
        }
    }
}

// MARK: - Project Picker Popover

struct ProjectPickerPopover: View {
    @Environment(DecompilerService.self) private var service
    @Environment(ProjectService.self) private var projectService
    @Binding var isPresented: Bool

    private var recentProjects: [Project] {
        Array(projectService.projects.prefix(5))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Recent")
                .font(.caption2.weight(.semibold))
                .foregroundStyle(.tertiary)
                .textCase(.uppercase)
                .tracking(0.6)
                .padding(.horizontal, 14)
                .padding(.top, 12)
                .padding(.bottom, 8)

            Divider()

            VStack(spacing: 0) {
                ForEach(recentProjects) { project in
                    let isCurrent = projectService.currentProject?.id == project.id
                    Button {
                        if !isCurrent {
                            projectService.openProject(project, in: service)
                        }
                        isPresented = false
                    } label: {
                        HStack(spacing: 10) {
                            Image(systemName: isCurrent ? "folder.fill" : "folder")
                                .font(.system(size: 11))
                                .foregroundStyle(isCurrent ? .primary : .secondary)
                                .frame(width: 16, alignment: .center)
                            Text(project.name)
                                .font(.callout)
                                .foregroundStyle(.primary)
                                .lineLimit(1)
                                .truncationMode(.middle)
                                .frame(maxWidth: .infinity, alignment: .leading)
                            if isCurrent {
                                Image(systemName: "checkmark")
                                    .font(.system(size: 10, weight: .semibold))
                                    .foregroundStyle(.tertiary)
                            }
                        }
                        .padding(.horizontal, 14)
                        .padding(.vertical, 9)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)

                    if project.id != recentProjects.last?.id {
                        Divider()
                            .padding(.horizontal, 14)
                    }
                }

                Divider()

                Button {
                    projectService.showingProjectManager = true
                    isPresented = false
                } label: {
                    HStack(spacing: 10) {
                        Image(systemName: "folder.badge.gearshape")
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                            .frame(width: 16, alignment: .center)
                        Text("Open Projects")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(.horizontal, 14)
                    .padding(.vertical, 9)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)

                Divider()
                    .padding(.horizontal, 14)

                Button {
                    projectService.closeProject(in: service)
                    isPresented = false
                } label: {
                    HStack(spacing: 10) {
                        Image(systemName: "rectangle.portrait.and.arrow.right")
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                            .frame(width: 16, alignment: .center)
                        Text("Close Project")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(.horizontal, 14)
                    .padding(.vertical, 9)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
            .padding(.bottom, 6)
        }
        .frame(width: 240)
    }
}
