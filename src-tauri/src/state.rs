use crate::cluster::{Clusterer, HashingEmbedder};
use crate::models::{Settings, Status};
use crate::store;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub clusterer: Mutex<Clusterer>,
    pub settings: Mutex<Settings>,
    pub status: Mutex<Status>,
    /// Сигнал «настройки изменились — переподключись».
    pub restart: Arc<Notify>,
}

impl AppState {
    pub fn new(data_dir: &std::path::Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let db = store::open(&data_dir.join("inbox.sqlite3"))?;

        let settings = store::load_settings(&db);

        // Стартуем с резервного эмбеддера — мгновенно; локальную модель
        // догрузим в фоне (см. lib.rs) и подменим.
        let mut clusterer = Clusterer::new(Box::new(HashingEmbedder::new()));
        for (topic, id, vec, count) in store::load_clusters(&db) {
            clusterer.load_centroid(&topic, id, vec, count);
        }

        Ok(AppState {
            db: Mutex::new(db),
            clusterer: Mutex::new(clusterer),
            settings: Mutex::new(settings),
            status: Mutex::new(Status::default()),
            restart: Arc::new(Notify::new()),
        })
    }
}
