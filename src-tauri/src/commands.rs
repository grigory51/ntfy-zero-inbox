use crate::models::*;
use crate::state::AppState;
use crate::store;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_channels(state: State<AppState>) -> Vec<ChannelDto> {
    let mut chans = store::channels(&state.db.lock().unwrap());
    // Подписанные топики показываем всегда, даже пока в них пусто.
    let topics = state.settings.lock().unwrap().topics.clone();
    for t in topics {
        if !chans.iter().any(|c| c.topic == t) {
            chans.push(ChannelDto {
                topic: t,
                total: 0,
                unread: 0,
                cluster_count: 0,
                last_title: None,
                last_body: None,
                last_time: None,
            });
        }
    }
    chans
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
    changed(&app);
}

#[tauri::command]
pub fn mark_cluster_read(app: AppHandle, state: State<AppState>, cluster_id: String) {
    store::mark_cluster_read(&state.db.lock().unwrap(), &cluster_id);
    changed(&app);
}

#[tauri::command]
pub fn mark_channel_read(app: AppHandle, state: State<AppState>, topic: String) {
    store::mark_channel_read(&state.db.lock().unwrap(), &topic);
    changed(&app);
}

#[tauri::command]
pub fn delete_message(app: AppHandle, state: State<AppState>, id: String) {
    store::delete_message(&state.db.lock().unwrap(), &id);
    changed(&app);
}

#[tauri::command]
pub fn delete_cluster(app: AppHandle, state: State<AppState>, cluster_id: String) {
    store::delete_cluster(&state.db.lock().unwrap(), &cluster_id);
    state.clusterer.lock().unwrap().remove_cluster(&cluster_id);
    changed(&app);
}

#[tauri::command]
pub fn delete_channel(app: AppHandle, state: State<AppState>, topic: String) {
    store::delete_channel(&state.db.lock().unwrap(), &topic);
    state.clusterer.lock().unwrap().remove_topic(&topic);
    changed(&app);
}

/// Уведомить фронт и перерисовать иконку в трее.
fn changed(app: &AppHandle) {
    let _ = app.emit("inbox-changed", ());
    crate::refresh_tray(app);
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
