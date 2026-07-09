import Foundation
import Observation

/// Настройки подключения. Хранятся в UserDefaults общего App Group,
/// чтобы при желании их мог прочитать и виджет.
///
/// Замечание по безопасности: токен для простоты лежит в UserDefaults.
/// Для продакшена вынеси в Keychain — это единственное место, где стоит.
@Observable
final class SettingsStore {
    private let defaults = UserDefaults(suiteName: AppConfig.appGroupID) ?? .standard

    var serverURL: String { didSet { defaults.set(serverURL, forKey: "serverURL") } }
    var topicsRaw: String { didSet { defaults.set(topicsRaw, forKey: "topicsRaw") } }
    var token: String { didSet { defaults.set(token, forKey: "token") } }
    /// Курсор для догрузки пропущенного после разрыва/отсутствия.
    /// "all" при первом запуске, дальше — время последнего сообщения.
    var sinceToken: String { didSet { defaults.set(sinceToken, forKey: "sinceToken") } }

    init() {
        let d = UserDefaults(suiteName: AppConfig.appGroupID) ?? .standard
        serverURL = d.string(forKey: "serverURL") ?? "https://ntfy.sh"
        topicsRaw = d.string(forKey: "topicsRaw") ?? ""
        token = d.string(forKey: "token") ?? ""
        sinceToken = d.string(forKey: "sinceToken") ?? "all"
    }

    /// Топики, разбитые по запятой/пробелу/переносу строки.
    var topicList: [String] {
        topicsRaw
            .split { $0 == "," || $0 == " " || $0 == "\n" || $0 == "\t" }
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
    }

    /// URL сервера, приведённый к виду: со схемой и без хвостового слэша.
    /// Спасает от частых опечаток («ntfy.host» без https, «.../» в конце).
    var normalizedServerURL: String {
        var s = serverURL.trimmingCharacters(in: .whitespacesAndNewlines)
        if !s.lowercased().hasPrefix("http://") && !s.lowercased().hasPrefix("https://") {
            s = "https://" + s
        }
        while s.hasSuffix("/") { s.removeLast() }
        return s
    }

    var trimmedToken: String {
        token.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var isConfigured: Bool {
        !normalizedServerURL.isEmpty
            && URL(string: normalizedServerURL) != nil
            && !topicList.isEmpty
    }
}
