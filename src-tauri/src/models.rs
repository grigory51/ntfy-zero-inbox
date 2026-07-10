use serde::{Deserialize, Serialize};

/// Входящее сообщение из стрима ntfy (`GET /<topic>/json`).
#[derive(Debug, Clone, Deserialize)]
pub struct NtfyMessage {
    pub id: String,
    pub time: i64,
    /// "open" | "keepalive" | "message" | "poll_request"
    pub event: String,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub priority: Option<i64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub click: Option<String>,
}

impl NtfyMessage {
    /// Текст для эмбеддинга: заголовок + тело.
    pub fn cluster_text(&self) -> String {
        let t = self.title.clone().unwrap_or_default();
        let b = self.message.clone().unwrap_or_default();
        format!("{t} {b}").trim().to_string()
    }
}

/// Настройки подключения (хранятся в SQLite).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server_url: String,
    pub token: String,
    pub topics: Vec<String>,
    /// Курсор догрузки пропущенного (ntfy `since`): "all" или unix-время.
    pub since: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            server_url: "https://ntfy.sh".into(),
            token: String::new(),
            topics: Vec::new(),
            since: "all".into(),
        }
    }
}

impl Settings {
    /// URL со схемой и без хвостового слэша.
    pub fn normalized_server(&self) -> String {
        let mut s = self.server_url.trim().to_string();
        let low = s.to_lowercase();
        if !low.starts_with("http://") && !low.starts_with("https://") {
            s = format!("https://{s}");
        }
        while s.ends_with('/') {
            s.pop();
        }
        s
    }

    pub fn is_configured(&self) -> bool {
        !self.normalized_server().is_empty() && !self.topics.is_empty()
    }
}

/// Статус подключения — отдаётся во фронтенд.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Status {
    pub connected: bool,
    pub error: Option<String>,
    /// Готова ли локальная модель эмбеддингов (иначе — резервный эмбеддер).
    pub model_ready: bool,
}

// ---- DTO для фронтенда ----

#[derive(Debug, Serialize)]
pub struct MessageDto {
    pub id: String,
    pub topic: String,
    pub cluster_id: String,
    pub title: Option<String>,
    pub body: String,
    pub priority: i64,
    pub tags: Vec<String>,
    pub time: i64,
    pub read: bool,
    pub click: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChannelDto {
    pub topic: String,
    pub total: i64,
    pub unread: i64,
    pub cluster_count: i64,
    pub last_body: Option<String>,
    pub last_title: Option<String>,
    pub last_time: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ClusterDto {
    pub id: String,
    pub topic: String,
    pub label: String,
    pub total: i64,
    pub unread: i64,
    pub last_time: i64,
    pub last_body: String,
}
