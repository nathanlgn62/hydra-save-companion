use std::sync::Arc;
use tauri::Manager;

use crate::models::game::GameProcessInfo;
use crate::models::watcher::ProcessMonitorState;

#[tauri::command]
pub fn set_monitored_games(app: tauri::AppHandle, games: Vec<GameProcessInfo>) {
    let state = app.state::<Arc<ProcessMonitorState>>();
    let mut monitored = state.monitored_games.lock().unwrap();
    *monitored = games;
}
