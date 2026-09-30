use rusty_leveldb::{LdbIterator, Options, DB};
use std::env;
use std::fs;

use crate::funcs::hydra::{copy_dir_all, get_hydra_db_dir};
use crate::models::game::HydraGame;

#[tauri::command]
pub fn get_installed_games() -> Result<Vec<HydraGame>, String> {
    let original_db_path = get_hydra_db_dir()?;

    if !original_db_path.exists() {
        return Err(format!("BDD Hydra introuvable : {:?}", original_db_path));
    }

    let temp_db_path = env::temp_dir().join("hydra_companion_db_copy");
    if temp_db_path.exists() {
        let _ = fs::remove_dir_all(&temp_db_path);
    }

    copy_dir_all(&original_db_path, &temp_db_path)
        .map_err(|e| format!("Erreur lors de la copie de la BDD : {}", e))?;

    let mut opt = Options::default();
    opt.create_if_missing = false;

    let mut db = DB::open(&temp_db_path, opt)
        .map_err(|e| format!("Impossible d'ouvrir LevelDB : {:?}", e))?;

    let mut games = Vec::new();
    let mut iter = db
        .new_iter()
        .map_err(|e| format!("Erreur d'itérateur : {:?}", e))?;

    while let Some((key, value)) = iter.next() {
        if key.starts_with(b"!games!") {
            if let Ok(value_str) = String::from_utf8(value) {
                if let Ok(game) = serde_json::from_str::<HydraGame>(&value_str) {
                    let is_active = !game.is_deleted.unwrap_or(false);
                    let is_steam_import = game.has_active_steam_import.unwrap_or(false);
                    let has_executable = game
                        .executable_path
                        .as_ref()
                        .map_or(false, |path| !path.trim().is_empty());

                    if is_active && has_executable && !is_steam_import {
                        games.push(game);
                    }
                }
            }
        }
    }

    let _ = fs::remove_dir_all(temp_db_path);

    Ok(games)
}
