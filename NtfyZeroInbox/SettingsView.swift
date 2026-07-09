import SwiftUI

struct SettingsView: View {
    @Bindable var settings: SettingsStore
    let client: NtfyClient

    var body: some View {
        Form {
            Section("Сервер") {
                TextField("URL сервера", text: $settings.serverURL,
                          prompt: Text("https://ntfy.example.com"))
                SecureField("Токен доступа (если сервер закрытый)", text: $settings.token)
            }

            Section("Топики") {
                TextField("Через запятую", text: $settings.topicsRaw,
                          prompt: Text("backups, ci, alerts"), axis: .vertical)
                    .lineLimit(2...4)
                Text("Каждый топик — отдельная категория/полка в инбоксе.")
                    .font(.caption).foregroundStyle(.secondary)
            }

            Section {
                Button("Применить и переподключиться") {
                    client.restart(with: settings)
                }
                HStack(spacing: 6) {
                    Circle()
                        .fill(client.isConnected ? Color.green : Color.orange)
                        .frame(width: 8, height: 8)
                    Text(client.isConnected ? "Подключено" : "Не подключено")
                        .font(.caption)
                    if let err = client.lastError {
                        Text("· \(err)")
                            .font(.caption).foregroundStyle(.red).lineLimit(1)
                    }
                }
            }
        }
        .formStyle(.grouped)
        .frame(width: 440, height: 360)
    }
}
