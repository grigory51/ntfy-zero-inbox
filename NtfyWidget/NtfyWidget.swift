import WidgetKit
import SwiftUI
import SwiftData

// MARK: - Данные для одного рендера виджета

struct InboxEntry: TimelineEntry {
    let date: Date
    let unreadCount: Int
    let latest: [Snapshot]

    struct Snapshot: Identifiable {
        let id: String
        let topic: String
        let text: String
    }
}

// MARK: - Провайдер

/// Виджет НЕ может держать живое соединение — он читает общий App-Group стор
/// по таймлайну. Приложение при каждом новом сообщении зовёт
/// `WidgetCenter.reloadAllTimelines()`, поэтому обновление почти мгновенное;
/// 15-минутный refresh — просто страховка.
struct Provider: TimelineProvider {
    func placeholder(in context: Context) -> InboxEntry {
        InboxEntry(date: .now, unreadCount: 3, latest: [
            .init(id: "1", topic: "backups", text: "Бэкап завершён"),
            .init(id: "2", topic: "ci", text: "Сборка упала")
        ])
    }

    func getSnapshot(in context: Context, completion: @escaping (InboxEntry) -> Void) {
        completion(fetch())
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<InboxEntry>) -> Void) {
        let next = Calendar.current.date(byAdding: .minute, value: 15, to: .now)
            ?? .now.addingTimeInterval(900)
        completion(Timeline(entries: [fetch()], policy: .after(next)))
    }

    private func fetch() -> InboxEntry {
        let context = ModelContext(AppConfig.makeModelContainer())

        let count = (try? context.fetchCount(
            FetchDescriptor<InboxItem>(predicate: #Predicate { !$0.isRead })
        )) ?? 0

        var descriptor = FetchDescriptor<InboxItem>(
            predicate: #Predicate { !$0.isRead },
            sortBy: [SortDescriptor(\.time, order: .reverse)]
        )
        descriptor.fetchLimit = 3
        let latest = ((try? context.fetch(descriptor)) ?? []).map { item in
            InboxEntry.Snapshot(
                id: item.id,
                topic: item.topic,
                text: (item.title?.isEmpty == false) ? item.title! : item.body
            )
        }

        return InboxEntry(date: .now, unreadCount: count, latest: latest)
    }
}

// MARK: - Вид

struct NtfyWidgetView: View {
    var entry: InboxEntry
    @Environment(\.widgetFamily) private var family

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Image(systemName: entry.unreadCount > 0 ? "tray.full.fill" : "tray")
                Text("Inbox").font(.headline)
                Spacer()
                Text("\(entry.unreadCount)")
                    .font(.title3.bold())
                    .foregroundStyle(entry.unreadCount > 0 ? Color.blue : Color.secondary)
            }

            if family == .systemSmall {
                Text(entry.unreadCount > 0 ? "непрочитанных" : "всё чисто")
                    .font(.caption).foregroundStyle(.secondary)
            } else if entry.latest.isEmpty {
                Text("Inbox zero 🎉").font(.caption).foregroundStyle(.secondary)
            } else {
                ForEach(entry.latest) { s in
                    HStack(spacing: 4) {
                        Text("#\(s.topic)").font(.caption2).foregroundStyle(.secondary)
                        Text(s.text).font(.caption).lineLimit(1)
                    }
                }
            }

            Spacer(minLength: 0)
        }
    }
}

// MARK: - Конфигурация

struct NtfyWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "NtfyInboxWidget", provider: Provider()) { entry in
            NtfyWidgetView(entry: entry)
                .containerBackground(.fill.tertiary, for: .widget)
        }
        .configurationDisplayName("Ntfy Inbox")
        .description("Непрочитанные уведомления от твоих автоматизаций")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

@main
struct NtfyWidgetBundle: WidgetBundle {
    var body: some Widget {
        NtfyWidget()
    }
}
