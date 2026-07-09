import SwiftUI
import SwiftData
import AppKit

struct MenuContentView: View {
    @Bindable var settings: SettingsStore
    let client: NtfyClient

    @Query(sort: \InboxItem.time, order: .reverse) private var allItems: [InboxItem]
    @State private var unreadOnly = true

    private var items: [InboxItem] {
        unreadOnly ? allItems.filter { !$0.isRead } : allItems
    }

    /// Группировка по топикам — те самые «полочки».
    private var grouped: [(topic: String, items: [InboxItem])] {
        Dictionary(grouping: items, by: \.topic)
            .map { (topic: $0.key, items: $0.value) }
            .sorted { $0.topic < $1.topic }
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
            Divider()
            footer
        }
        .frame(width: 380, height: 500)
    }

    private var header: some View {
        VStack(spacing: 4) {
            HStack {
                Circle()
                    .fill(client.isConnected ? Color.green : Color.orange)
                    .frame(width: 8, height: 8)
                Text(client.isConnected ? "На связи" : "Переподключение…")
                    .font(.caption).foregroundStyle(.secondary)
                Spacer()
                Picker("", selection: $unreadOnly) {
                    Text("Непрочит.").tag(true)
                    Text("Все").tag(false)
                }
                .pickerStyle(.segmented)
                .fixedSize()
            }
            if let err = client.lastError, !client.isConnected {
                Text(err)
                    .font(.caption2).foregroundStyle(.red)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .lineLimit(2)
            }
        }
        .padding(8)
    }

    @ViewBuilder private var content: some View {
        if !settings.isConfigured {
            hint("Открой настройки и укажи сервер и топики", systemImage: "gearshape")
        } else if items.isEmpty {
            hint(unreadOnly ? "Inbox zero 🎉" : "Пока пусто", systemImage: "tray")
        } else {
            List {
                ForEach(grouped, id: \.topic) { group in
                    Section("#\(group.topic)") {
                        ForEach(group.items) { item in
                            InboxRow(
                                item: item,
                                onRead: { client.markRead(item) },
                                onDelete: { client.delete(item) }
                            )
                        }
                    }
                }
            }
            .listStyle(.inset)
        }
    }

    private var footer: some View {
        HStack {
            Button {
                client.markAllRead()
            } label: {
                Label("Прочитать все", systemImage: "checkmark.circle")
            }
            .disabled(client.unreadCount == 0)

            Spacer()
            Text("\(client.unreadCount) непрочит.")
                .font(.caption).foregroundStyle(.secondary)
            Spacer()

            // SettingsLink надёжнее openSettings() из MenuBarExtra. У agent-app
            // окно не выходит вперёд, пока политика .accessory — временно делаем
            // .regular (обратно вернём при закрытии окна, см. SettingsView).
            SettingsLink {
                Image(systemName: "gearshape")
            }
            .simultaneousGesture(TapGesture().onEnded {
                NSApp.setActivationPolicy(.regular)
                NSApp.activate(ignoringOtherApps: true)
            })
            .help("Настройки")

            Button { NSApp.terminate(nil) } label: { Image(systemName: "power") }
                .help("Выйти")
        }
        .buttonStyle(.borderless)
        .padding(8)
    }

    private func hint(_ text: String, systemImage: String) -> some View {
        VStack(spacing: 10) {
            Image(systemName: systemImage)
                .font(.largeTitle).foregroundStyle(.secondary)
            Text(text).foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct InboxRow: View {
    let item: InboxItem
    let onRead: () -> Void
    let onDelete: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Circle()
                .fill(item.isRead ? Color.clear : Color.blue)
                .frame(width: 8, height: 8)
                .padding(.top, 5)

            VStack(alignment: .leading, spacing: 2) {
                if let title = item.title, !title.isEmpty {
                    Text(title).font(.system(size: 13, weight: .semibold))
                }
                Text(item.body)
                    .font(.system(size: 12))
                    .foregroundStyle((item.title?.isEmpty == false) ? .secondary : .primary)
                    .lineLimit(4)

                HStack(spacing: 6) {
                    Text(item.time, style: .relative).foregroundStyle(.tertiary)
                    ForEach(item.tags, id: \.self) { tag in
                        Text(tag)
                            .padding(.horizontal, 5).padding(.vertical, 1)
                            .background(.quaternary, in: Capsule())
                    }
                }
                .font(.caption2)
            }

            Spacer(minLength: 4)

            if !item.isRead {
                Button(action: onRead) { Image(systemName: "checkmark") }
                    .buttonStyle(.borderless)
                    .help("Отметить обработанным")
            }
        }
        .padding(.vertical, 2)
        .contentShape(Rectangle())
        .contextMenu {
            if !item.isRead {
                Button("Отметить обработанным", action: onRead)
            }
            if let raw = item.clickURL, let url = URL(string: raw) {
                Button("Открыть ссылку") { NSWorkspace.shared.open(url) }
            }
            Button("Удалить", role: .destructive, action: onDelete)
        }
    }
}
