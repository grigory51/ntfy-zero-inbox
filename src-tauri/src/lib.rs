mod cluster;
mod commands;
mod models;
mod ntfy;
mod state;
mod store;

use state::AppState;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_positioner::{Position, WindowExt};

/// Монохромная template-иконка (macOS сам перекрашивает под тему панели).
const TRAY_MONO: &[u8] = include_bytes!("../icons/tray-mono.png");
/// Иконка с красной точкой — когда есть непрочитанное.
const TRAY_UNREAD: &[u8] = include_bytes!("../icons/tray-unread.png");

/// Перерисовать иконку в трее по числу непрочитанных (на главном потоке).
pub(crate) fn refresh_tray(app: &tauri::AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let unread = {
            let state = handle.state::<AppState>();
            let db = state.db.lock().unwrap();
            store::total_unread(&db)
        };
        if let Some(tray) = handle.tray_by_id("main-tray") {
            let bytes = if unread > 0 { TRAY_UNREAD } else { TRAY_MONO };
            if let Ok(img) = Image::from_bytes(bytes) {
                let _ = tray.set_icon(Some(img));
                // Красную точку видно только у non-template иконки.
                #[cfg(target_os = "macos")]
                let _ = tray.set_icon_as_template(unread == 0);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_channels,
            commands::get_clusters,
            commands::get_messages,
            commands::mark_read,
            commands::mark_cluster_read,
            commands::mark_channel_read,
            commands::delete_message,
            commands::delete_cluster,
            commands::delete_channel,
            commands::get_settings,
            commands::get_status,
            commands::save_settings,
        ])
        .setup(|app| {
            // Хранилище + состояние.
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let state = AppState::new(&data_dir)?;
            app.manage(state);

            // macOS: agent-режим без иконки в Dock.
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            build_tray(app)?;
            refresh_tray(&app.handle());

            // Прятать попап при потере фокуса.
            if let Some(win) = app.get_webview_window("main") {
                let w = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = w.hide();
                    }
                });
            }

            // Фоновая подписка на ntfy.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                ntfy::run(handle).await;
            });

            // Догрузка локальной модели эмбеддингов в фоне (не блокируя старт).
            #[cfg(not(any(target_os = "ios", target_os = "android")))]
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || match cluster::FastEmbedder::try_new() {
                    Ok(fe) => {
                        let state = handle.state::<AppState>();
                        state.clusterer.lock().unwrap().set_embedder(Box::new(fe));
                        let snap = {
                            let mut st = state.status.lock().unwrap();
                            st.model_ready = true;
                            st.clone()
                        };
                        let _ = handle.emit("status", snap);
                        println!("[cluster] локальная модель эмбеддингов готова");
                    }
                    Err(e) => {
                        eprintln!("[cluster] модель не загрузилась, работаю на резервном эмбеддере: {e}");
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let open_i = MenuItem::with_id(app, "open", "Открыть", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Выйти", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

    let icon = Image::from_bytes(TRAY_MONO).expect("не удалось прочитать иконку трея");

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_popup(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            tauri_plugin_positioner::on_tray_event(app, &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_popup(app);
            }
        })
        .build(app)?;
    Ok(())
}

fn show_popup(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.move_window(Position::TrayBottomCenter);
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn toggle_popup(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            show_popup(app);
        }
    }
}
