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

#[tauri::command]
pub fn setup_demo_environment() -> Result<Vec<HydraGame>, String> {
    let base_dir = env::temp_dir().join("hydra_companion_demo_saves");

    // 1. Elden Ring
    let er_dir = base_dir.join("Elden Ring").join("76561198000000000");
    let _ = fs::create_dir_all(&er_dir);
    let er_save_file = er_dir.join("ER0000.sl2");
    if !er_save_file.exists() {
        let _ = fs::write(&er_save_file, b"MOCK_ELDEN_RING_SAVE_DATA_DEMO");
    }

    // 2. Hollow Knight
    let hk_dir = base_dir.join("Hollow Knight");
    let _ = fs::create_dir_all(&hk_dir);
    let hk_save_file = hk_dir.join("user1.dat");
    if !hk_save_file.exists() {
        let _ = fs::write(&hk_save_file, b"MOCK_HOLLOW_KNIGHT_SAVE_DATA_DEMO");
    }

    // 3. Cyberpunk 2077
    let cp_dir = base_dir.join("Cyberpunk 2077").join("ManualSave-0");
    let _ = fs::create_dir_all(&cp_dir);
    let cp_save_file = cp_dir.join("sav.dat");
    if !cp_save_file.exists() {
        let _ = fs::write(&cp_save_file, b"MOCK_CYBERPUNK_SAVE_DATA_DEMO");
    }

    let demo_games = vec![
        HydraGame {
            title: "ELDEN RING".to_string(),
            object_id: "1245620".to_string(),
            shop: Some("steam".to_string()),
            executable_path: Some("/tmp/eldenring.exe".to_string()),
            is_deleted: Some(false),
            icon_url: Some("https://cdn.cloudflare.steamstatic.com/steam/apps/1245620/library_600x900_2x.jpg".to_string()),
            has_active_steam_import: Some(false),
        },
        HydraGame {
            title: "Hollow Knight".to_string(),
            object_id: "367520".to_string(),
            shop: Some("steam".to_string()),
            executable_path: Some("/tmp/hollow_knight.exe".to_string()),
            is_deleted: Some(false),
            icon_url: Some("https://cdn.cloudflare.steamstatic.com/steam/apps/367520/library_600x900_2x.jpg".to_string()),
            has_active_steam_import: Some(false),
        },
        HydraGame {
            title: "Cyberpunk 2077".to_string(),
            object_id: "1091500".to_string(),
            shop: Some("steam".to_string()),
            executable_path: Some("/tmp/Cyberpunk2077.exe".to_string()),
            is_deleted: Some(false),
            icon_url: Some("https://cdn.cloudflare.steamstatic.com/steam/apps/1091500/library_600x900_2x.jpg".to_string()),
            has_active_steam_import: Some(false),
        },
    ];

    Ok(demo_games)
}

