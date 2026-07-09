import SwiftUI
import SwiftData

@main
struct NtfyZeroInboxApp: App {
    private let container: ModelContainer
    @State private var settings: SettingsStore
    @State private var client: NtfyClient

    init() {
        let container = AppConfig.makeModelContainer()
        let settings = SettingsStore()
        let client = NtfyClient(modelContainer: container)
        client.configure(with: settings)
        client.start() // agent-app резидентен → стрим живёт с момента запуска

        self.container = container
        _settings = State(initialValue: settings)
        _client = State(initialValue: client)
    }

    var body: some Scene {
        MenuBarExtra {
            MenuContentView(settings: settings, client: client)
                .modelContainer(container)
        } label: {
            // Бейдж с числом необработанных прямо в menu bar.
            if client.unreadCount > 0 {
                Image(systemName: "tray.full.fill")
                Text("\(client.unreadCount)")
            } else {
                Image(systemName: "tray")
            }
        }
        .menuBarExtraStyle(.window)

        Settings {
            SettingsView(settings: settings, client: client)
        }
    }
}
