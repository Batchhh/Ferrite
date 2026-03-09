import AppKit

// MARK: - Export & Persistence

extension ProjectService {
    /// Export the currently selected type or member as a `.cs` file via a save panel.
    func exportCode(selection: Selection?, in service: DecompilerService) {
        var lines: [CodeLine] = []
        var fileName = "code.cs"

        switch selection {
        case .type(let assemblyId, let token):
            if let type_ = service.findType(assemblyId: assemblyId, token: token) {
                lines = generateTypeCode(type_)
                fileName = "\(type_.name).cs"
            }
        case .member(let assemblyId, let typeToken, let memberToken):
            if let type_ = service.findType(assemblyId: assemblyId, token: typeToken),
               let member = service.findMember(
                   assemblyId: assemblyId, typeToken: typeToken, memberToken: memberToken
               ) {
                lines = generateMemberCode(member, declaringType: type_)
                fileName = "\(type_.name).\(member.name).cs"
            }
        default:
            return
        }

        let plainText = lines.map { $0.tokens.map(\.text).joined() }.joined(separator: "\n")

        let panel = NSSavePanel()
        panel.allowedContentTypes = [.plainText]
        panel.nameFieldStringValue = fileName
        panel.canCreateDirectories = true

        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try plainText.write(to: url, atomically: true, encoding: .utf8)
        } catch {
            lastError = "Failed to export code: \(error.localizedDescription)"
        }
    }

    /// Export the currently selected type as a fields-only C++ `.h` header via a save panel.
    func exportHeader(selection: Selection?, in service: DecompilerService) {
        guard case .type(let assemblyId, let token) = selection,
              let type_ = service.findType(assemblyId: assemblyId, token: token) else { return }

        let text = generateHeaderExport(rootType: type_, assemblyId: assemblyId, service: service)
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.init(filenameExtension: "h")!]
        panel.nameFieldStringValue = "\(type_.name).h"
        panel.canCreateDirectories = true

        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try text.write(to: url, atomically: true, encoding: .utf8)
        } catch {
            lastError = "Failed to export header: \(error.localizedDescription)"
        }
    }

    func showOpenPanel(in service: DecompilerService) {
        let panel = NSOpenPanel()
        panel.title = "Open .NET Assembly"
        panel.allowedContentTypes = [
            .init(filenameExtension: "dll")!,
            .init(filenameExtension: "exe")!,
        ]
        panel.allowsMultipleSelection = true
        panel.canChooseDirectories = false
        if panel.runModal() == .OK {
            if panel.urls.count >= 20 {
                service.loadAssemblies(urls: panel.urls)
                if let project = currentProject,
                   let idx = projects.firstIndex(where: { $0.id == project.id }) {
                    for url in panel.urls {
                        let path = url.path
                        if !projects[idx].dllPaths.contains(path) {
                            projects[idx].dllPaths.append(path)
                        }
                    }
                    currentProject = projects[idx]
                    save()
                }
            } else {
                for url in panel.urls {
                    addAssembly(url: url, in: service)
                }
            }
        }
    }

    /// Encode and atomically write the project store to disk.
    func save() {
        do {
            let store = ProjectStore(projects: projects, tags: availableTags)
            let data = try JSONEncoder().encode(store)
            try data.write(to: storageURL, options: .atomic)
        } catch {
            print("ProjectService: failed to save: \(error)")
        }
    }

    /// Load projects and tags from disk, falling back to a legacy `[Project]` format if needed.
    func load() {
        let data: Data
        do {
            data = try Data(contentsOf: storageURL)
        } catch {
            // No saved data yet — not an error on first launch.
            return
        }
        do {
            let store = try JSONDecoder().decode(ProjectStore.self, from: data)
            projects = store.projects.sorted { $0.lastOpenedAt > $1.lastOpenedAt }
            availableTags = store.tags
        } catch {
            // Try legacy format before reporting error.
            do {
                let decoded = try JSONDecoder().decode([Project].self, from: data)
                projects = decoded.sorted { $0.lastOpenedAt > $1.lastOpenedAt }
                availableTags = []
            } catch {
                lastError = "Failed to load projects: \(error.localizedDescription)"
            }
        }
    }
}
