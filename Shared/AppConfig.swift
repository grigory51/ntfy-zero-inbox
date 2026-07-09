import Foundation
import SwiftData

/// Общие константы и фабрика хранилища, разделяемого между приложением и виджетом.
enum AppConfig {
    /// Должен совпадать с App Group в entitlements обоих таргетов (см. project.yml).
    static let appGroupID = "group.com.grigory.ntfy-zero-inbox"

    /// SwiftData-контейнер, физически лежащий в общем контейнере App Group,
    /// чтобы и menu-bar приложение (пишет), и виджет (читает) видели одни данные.
    static func makeModelContainer() -> ModelContainer {
        let schema = Schema([InboxItem.self])
        let config = ModelConfiguration(
            schema: schema,
            isStoredInMemoryOnly: false,
            groupContainer: .identifier(appGroupID)
        )
        do {
            return try ModelContainer(for: schema, configurations: config)
        } catch {
            fatalError("Не удалось создать ModelContainer: \(error)")
        }
    }
}
