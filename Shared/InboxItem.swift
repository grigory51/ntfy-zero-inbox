import Foundation
import SwiftData

/// Одно уведомление в инбоксе. `isRead` — то, чего нет у готовых ntfy-клиентов:
/// именно оно даёт zero-inbox workflow (отметил обработанным → ушло из очереди).
@Model
final class InboxItem {
    /// id сообщения из ntfy — уникальный, чтобы не дублировать при реконнекте.
    @Attribute(.unique) var id: String
    var topic: String
    var title: String?
    var body: String
    var priority: Int
    var tags: [String]
    var time: Date
    var isRead: Bool
    var clickURL: String?

    init(
        id: String,
        topic: String,
        title: String?,
        body: String,
        priority: Int,
        tags: [String],
        time: Date,
        isRead: Bool = false,
        clickURL: String? = nil
    ) {
        self.id = id
        self.topic = topic
        self.title = title
        self.body = body
        self.priority = priority
        self.tags = tags
        self.time = time
        self.isRead = isRead
        self.clickURL = clickURL
    }
}
