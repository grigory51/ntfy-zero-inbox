use crate::models::{NtfyMessage, Settings};
use crate::state::AppState;
use crate::store;
use futures_util::StreamExt;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Фоновый цикл: подписка на ntfy с переподключением. Просыпается на сигнал
/// `restart`, когда меняются настройки.
pub async fn run(app: AppHandle) {
    loop {
        let (cfg, notify) = {
            let state = app.state::<AppState>();
            (state.settings.lock().unwrap().clone(), state.restart.clone())
        };

        if !cfg.is_configured() {
            set_status(&app, false, Some("Не настроено: укажи сервер и топики".into()));
            notify.notified().await;
            continue;
        }

        tokio::select! {
            res = subscribe(&app, &cfg) => {
                if let Err(e) = res {
                    set_status(&app, false, Some(e.to_string()));
                }
                // backoff перед реконнектом, но прерываемся на смену настроек
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(3)) => {}
                    _ = notify.notified() => {}
                }
            }
            _ = notify.notified() => { /* настройки изменились — на новый круг */ }
        }
    }
}

async fn subscribe(app: &AppHandle, cfg: &Settings) -> anyhow::Result<()> {
    let base = cfg.normalized_server();
    let topics = cfg.topics.join(",");
    let since = if cfg.since.is_empty() { "all".into() } else { cfg.since.clone() };
    let url = format!("{base}/{topics}/json?since={since}");

    let client = reqwest::Client::builder().build()?;
    let mut req = client.get(&url);
    if !cfg.token.trim().is_empty() {
        req = req.bearer_auth(cfg.token.trim());
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {} от {} — проверь адрес/токен", resp.status().as_u16(), base);
    }
    set_status(app, true, None);

    // ntfy отдаёт NDJSON — по сообщению на строку.
    let mut stream = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.next().await {
        buf.extend_from_slice(&chunk?);
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=pos).collect();
            let line = &line[..line.len() - 1];
            if line.is_empty() {
                continue;
            }
            if let Ok(msg) = serde_json::from_slice::<NtfyMessage>(line) {
                handle_message(app, msg);
            }
        }
    }
    Ok(())
}

/// Синхронная обработка одного сообщения: дедуп → эмбеддинг+кластер → запись.
fn handle_message(app: &AppHandle, msg: NtfyMessage) {
    if msg.event != "message" {
        return; // open / keepalive / poll_request
    }
    let state = app.state::<AppState>();
    let next_since = (msg.time + 1).to_string();

    // Дедуп до кластеризации, иначе повторное сообщение раздует счётчики.
    let dup = store::message_exists(&state.db.lock().unwrap(), &msg.id);
    if dup {
        state.settings.lock().unwrap().since = next_since.clone();
        store::save_since(&state.db.lock().unwrap(), &next_since);
        return;
    }

    let text = msg.cluster_text();
    let assign = state.clusterer.lock().unwrap().assign(&msg.topic, &text);

    {
        let db = state.db.lock().unwrap();
        let _ = store::insert_message(&db, &msg, &assign.cluster_id);
        // label обновляется только при вставке нового кластера (ON CONFLICT его не трогает).
        let _ = store::upsert_cluster(
            &db,
            &assign.cluster_id,
            &msg.topic,
            &text,
            &assign.centroid,
            assign.count,
            msg.time,
        );
        store::save_since(&db, &next_since);
    }
    state.settings.lock().unwrap().since = next_since;

    let _ = app.emit("inbox-changed", ());
}

fn set_status(app: &AppHandle, connected: bool, error: Option<String>) {
    let state = app.state::<AppState>();
    let snap = {
        let mut st = state.status.lock().unwrap();
        st.connected = connected;
        st.error = error;
        st.clone()
    };
    let _ = app.emit("status", snap);
}
