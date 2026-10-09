mod commands;
mod funcs;
mod models;
mod utils;

use funcs::watcher::start_process_watcher;

use commands::cloud::download_game_save_from_drive;
use commands::cloud::login_cloud;
use commands::cloud::upload_game_save_to_drive;
use commands::game::check_game_sync_status;
use commands::game::get_game_save_info;
use commands::game::get_steam_cover;
use commands::hydra::get_installed_games;
use commands::hydra::setup_demo_environment;
use commands::utils::open_folder;
use commands::watcher::set_monitored_games;

use crate::models::watcher::ProcessMonitorState;

use std::env;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_installed_games,
            setup_demo_environment,
            get_game_save_info,
            check_game_sync_status,
            open_folder,
            login_cloud,
            upload_game_save_to_drive,
            set_monitored_games,
            download_game_save_from_drive,
            get_steam_cover
        ])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Ouvrir", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let monitor_state = Arc::new(ProcessMonitorState {
                monitored_games: Mutex::new(vec![]),
                current_running_game: Mutex::new(None),
            });
            app.manage(monitor_state.clone());

            let handle = app.handle().clone();
            start_process_watcher(handle, monitor_state);

            // Préchargement asynchrone en arrière-plan du manifest Ludasavi
            // pour éliminer tout temps d'attente lors de la première requête frontend
            std::thread::spawn(|| {
                let _ = funcs::ludasavi::get_cached_manifest();
            });

            Ok(())
        })
        .plugin(tauri_plugin_oauth::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .run(tauri::generate_context!())
        .expect("erreur lors de l'exécution de l'application tauri");
}
