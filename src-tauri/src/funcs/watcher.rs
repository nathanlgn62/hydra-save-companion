use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::System;
use tauri::{AppHandle, Emitter};

use crate::models::game::GameProcessInfo;
use crate::models::watcher::ProcessMonitorState;

pub fn start_process_watcher(app_handle: AppHandle, state: Arc<ProcessMonitorState>) {
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        
        loop {
            // Correction 1 : refresh sans argument
            sys.refresh_processes();

            let monitored = state.monitored_games.lock().unwrap().clone();
            let mut current_running = state.current_running_game.lock().unwrap();

            let mut detected_game: Option<String> = None;

            for process in sys.processes().values() {
                // Correction 2 : .to_string() au lieu de .to_string_lossy()
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
                    let _ = app_handle.emit("game-closed", old_game_title);
                }
                *current_running = detected_game;
            }

            std::thread::sleep(Duration::from_secs(3));
        }
    });
}