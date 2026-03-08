import Foundation

@MainActor @Observable
final class UpdateService {
    var latestVersion: String?
    var releaseURL: URL?
    var isDismissed = false

    var isUpdateAvailable: Bool {
        guard let latest = latestVersion else { return false }
        return isNewer(latest, than: currentVersion)
    }

    private var hasChecked = false

    func checkForUpdates() async {
        guard !hasChecked else { return }
        hasChecked = true

        let key = "lastUpdateCheck"
        if let last = UserDefaults.standard.object(forKey: key) as? Date,
           Date().timeIntervalSince(last) < 86400 {
            return
        }

        guard let url = URL(string: "https://api.github.com/repos/Batchhh/Ferrite/releases/latest") else { return }

        do {
            var request = URLRequest(url: url)
            request.setValue("application/vnd.github+json", forHTTPHeaderField: "Accept")
            let (data, _) = try await URLSession.shared.data(for: request)
            let release = try JSONDecoder().decode(GitHubRelease.self, from: data)

            let tag = release.tagName.hasPrefix("v")
                ? String(release.tagName.dropFirst())
                : release.tagName

            if let htmlURL = URL(string: release.htmlUrl) {
                latestVersion = tag
                releaseURL = htmlURL
            }

            UserDefaults.standard.set(Date(), forKey: key)
        } catch {
            // Silent failure — best-effort feature
        }
    }

    func dismiss() {
        isDismissed = true
    }

    // MARK: - Private

    private var currentVersion: String {
        Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.0.0"
    }

    private func isNewer(_ remote: String, than local: String) -> Bool {
        let r = remote.split(separator: ".").compactMap { Int($0) }
        let l = local.split(separator: ".").compactMap { Int($0) }
        for i in 0..<max(r.count, l.count) {
            let rv = i < r.count ? r[i] : 0
            let lv = i < l.count ? l[i] : 0
            if rv > lv { return true }
            if rv < lv { return false }
        }
        return false
    }
}

private struct GitHubRelease: Codable {
    let tagName: String
    let htmlUrl: String

    enum CodingKeys: String, CodingKey {
        case tagName = "tag_name"
        case htmlUrl = "html_url"
    }
}
