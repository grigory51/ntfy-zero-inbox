import Foundation

/// Формат одного JSON-сообщения из стрима ntfy (`GET /<topic>/json`).
/// Поля: https://docs.ntfy.sh/subscribe/api/#json-message-format
struct NtfyMessage: Decodable {
    let id: String
    let time: Int
    /// "open" | "keepalive" | "message" | "poll_request" — нас интересует "message".
    let event: String
    let topic: String
    let message: String?
    let title: String?
    let priority: Int?
    let tags: [String]?
    let click: String?
}
