use crate::models::game::GameProcessInfo;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub struct ProcessMonitorState {
    pub monitored_games: Mutex<Vec<GameProcessInfo>>,
    pub current_running_game: Mutex<Option<String>>,
}