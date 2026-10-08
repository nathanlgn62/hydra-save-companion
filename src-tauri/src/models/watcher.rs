use crate::models::game::GameProcessInfo;
use std::sync::Mutex;

pub struct ProcessMonitorState {
    pub monitored_games: Mutex<Vec<GameProcessInfo>>,
    pub current_running_game: Mutex<Option<String>>,
}