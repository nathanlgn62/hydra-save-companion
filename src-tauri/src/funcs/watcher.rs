use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tauri::{AppHandle, Emitter};

use crate::models::watcher::ProcessMonitorState;

#[derive(Clone, Serialize)]
pub struct GameClosedPayload {
    pub title: String,
    pub save_path: Option<String>,
}

pub fn start_process_watcher(app_handle: AppHandle, state: Arc<ProcessMonitorState>) {
    std::thread::spawn(move || {
        let mut sys = System::new_all();

        loop {
            sys.refresh_processes();

            let monitored = state.monitored_games.lock().unwrap().clone();
            let mut current_running = state.current_running_game.lock().unwrap();

            let mut detected_game: Option<String> = None;

            for process in sys.processes().values() {
                let proc_name = process.name().to_string().to_lowercase();
                for game in &monitored {
                    if proc_name == game.executable_name.to_lowercase() {
                        detected_game = Some(game.title.clone());
                        break;
                    }
                }
                if detected_game.is_some() {
                    break;
                }
            }

            if *current_running != detected_game {
                if let Some(ref game_title) = detected_game {
                    let _ = app_handle.emit("game-started", game_title);
                } else if let Some(ref old_game_title) = *current_running {
                    // Retrouver le save_path associé au jeu fermé
                    let save_path = monitored
                        .iter()
                        .find(|g| g.title == *old_game_title)
                        .and_then(|g| g.save_path.clone());

                    let payload = GameClosedPayload {
                        title: old_game_title.clone(),
                        save_path,
                    };

                    let _ = app_handle.emit("game-closed", payload);
                }
                *current_running = detected_game;
            }

            std::thread::sleep(Duration::from_secs(3));
        }
    });
}
