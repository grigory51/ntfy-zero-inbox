import Foundation
import SwiftData
import WidgetKit

/// Держит живое подключение к ntfy, складывает уведомления в SwiftData
/// и централизует все мутации инбокса (отметка обработанным / удаление),
/// заодно обновляя виджет.
@MainActor
@Observable
final class NtfyClient {
    private let context: ModelContext
    private var settings: SettingsStore?
    private var task: Task<Void, Never>?

    /// Отдельная сессия с длинным таймаутом — стрим держится часами,
    /// keepalive от ntfy (~45с) сбрасывает таймер простоя.
    private let session: URLSession = {
        let cfg = URLSessionConfiguration.default
        cfg.timeoutIntervalForRequest = 3600
        cfg.waitsForConnectivity = true
        return URLSession(configuration: cfg)
    }()

    var isConnected = false
    var unreadCount = 0
    var lastError: String?

    init(modelContainer: ModelContainer) {
        self.context = modelContainer.mainContext
        refreshUnreadCount()
    }

    func configure(with settings: SettingsStore) {
        self.settings = settings
    }

    // MARK: - Жизненный цикл подключения

    func start() {
        guard task == nil, let settings, settings.isConfigured else { return }
        task = Task { [weak self] in await self?.runLoop(settings: settings) }
    }

    func stop() {
        task?.cancel()
        task = nil
        isConnected = false
    }

    /// Применить новые настройки и переподключиться.
    func restart(with settings: SettingsStore) {
        self.settings = settings
        stop()
        start()
    }

    /// Цикл с экспоненциальным backoff: разорвалось → подождали → снова,
    /// с догрузкой пропущенного через `since`.
    private func runLoop(settings: SettingsStore) async {
        var backoff: UInt64 = 1
        while !Task.isCancelled {
            do {
                try await subscribe(settings: settings)
                backoff = 1
            } catch is CancellationError {
                break
            } catch {
                lastError = error.localizedDescription
                isConnected = false
            }
            if Task.isCancelled { break }
            try? await Task.sleep(nanoseconds: backoff * 1_000_000_000)
            backoff = min(backoff * 2, 30)
        }
    }

    private func subscribe(settings: SettingsStore) async throws {
        guard var comps = URLComponents(string: settings.serverURL) else {
            throw URLError(.badURL)
        }
        let topics = settings.topicList.joined(separator: ",")
        comps.path = "/\(topics)/json"
        comps.queryItems = [URLQueryItem(name: "since", value: settings.sinceToken)]
        guard let url = comps.url else { throw URLError(.badURL) }

        var request = URLRequest(url: url)
        if !settings.token.isEmpty {
            request.setValue("Bearer \(settings.token)", forHTTPHeaderField: "Authorization")
        }

        let (bytes, response) = try await session.bytes(for: request)
        guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
            throw URLError(.badServerResponse)
        }
        isConnected = true
        lastError = nil

        // ntfy отдаёт NDJSON — по одному сообщению на строку.
        for try await line in bytes.lines {
            if Task.isCancelled { break }
            guard let data = line.data(using: .utf8),
                  let msg = try? JSONDecoder().decode(NtfyMessage.self, from: data)
            else { continue }
            handle(msg, settings: settings)
        }
    }

    private func handle(_ msg: NtfyMessage, settings: SettingsStore) {
        guard msg.event == "message" else { return } // open/keepalive/poll игнорим
        insert(msg)
        // Следующий раз резюмируем сразу после этого сообщения.
        settings.sinceToken = String(msg.time + 1)
    }

    // MARK: - Мутации инбокса

    private func insert(_ msg: NtfyMessage) {
        let id = msg.id
        let already = (try? context.fetchCount(
            FetchDescriptor<InboxItem>(predicate: #Predicate { $0.id == id })
        )) ?? 0
        guard already == 0 else { return }

        context.insert(InboxItem(
            id: msg.id,
            topic: msg.topic,
            title: msg.title,
            body: msg.message ?? "",
            priority: msg.priority ?? 3,
            tags: msg.tags ?? [],
            time: Date(timeIntervalSince1970: TimeInterval(msg.time)),
            clickURL: msg.click
        ))
        save()
    }

    func markRead(_ item: InboxItem) {
        item.isRead = true
        save()
    }

    func markAllRead() {
        let unread = (try? context.fetch(
            FetchDescriptor<InboxItem>(predicate: #Predicate { !$0.isRead })
        )) ?? []
        unread.forEach { $0.isRead = true }
        save()
    }

    func delete(_ item: InboxItem) {
        context.delete(item)
        save()
    }

    private func save() {
        try? context.save()
        refreshUnreadCount()
        WidgetCenter.shared.reloadAllTimelines()
    }

    private func refreshUnreadCount() {
        unreadCount = (try? context.fetchCount(
            FetchDescriptor<InboxItem>(predicate: #Predicate { !$0.isRead })
        )) ?? 0
    }
}
