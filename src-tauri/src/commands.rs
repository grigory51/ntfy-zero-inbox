use crate::models::*;
use crate::state::AppState;
use crate::store;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_channels(state: State<AppState>) -> Vec<ChannelDto> {
    store::channels(&state.db.lock().unwrap())
}

#[tauri::command]
pub fn get_clusters(state: State<AppState>, topic: String) -> Vec<ClusterDto> {
    store::clusters_for(&state.db.lock().unwrap(), &topic)
}

#[tauri::command]
pub fn get_messages(state: State<AppState>, cluster_id: String) -> Vec<MessageDto> {
    store::messages_for(&state.db.lock().unwrap(), &cluster_id)
}

#[tauri::command]
pub fn mark_read(app: AppHandle, state: State<AppState>, id: String) {
    store::mark_read(&state.db.lock().unwrap(), &id);
    let _ = app.emit("inbox-changed", ());
}

#[tauri::command]
pub fn mark_cluster_read(app: AppHandle, state: State<AppState>, cluster_id: String) {
    store::mark_cluster_read(&state.db.lock().unwrap(), &cluster_id);
    let _ = app.emit("inbox-changed", ());
}

#[tauri::command]
pub fn mark_channel_read(app: AppHandle, state: State<AppState>, topic: String) {
    store::mark_channel_read(&state.db.lock().unwrap(), &topic);
    let _ = app.emit("inbox-changed", ());
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> Status {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<AppState>,
    server_url: String,
    token: String,
    topics: Vec<String>,
) -> Result<(), String> {
    {
        let mut s = state.settings.lock().unwrap();
        s.server_url = server_url;
        s.token = token;
        s.topics = topics;
        let db = state.db.lock().unwrap();
        store::save_settings(&db, &s).map_err(|e| e.to_string())?;
    }
    // Разбудить цикл ntfy — переподключится с новыми настройками.
    state.restart.notify_one();
    let _ = app.emit("inbox-changed", ());
    Ok(())
}
