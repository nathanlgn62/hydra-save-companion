mod funcs;
mod models;

use funcs::cloud::get_or_create_drive_folder;
use funcs::cloud::get_valid_access_token;
use funcs::hydra::copy_dir_all;
use funcs::hydra::expand_path;
use funcs::hydra::get_candidate_paths;
use funcs::hydra::get_hydra_db_dir;
use funcs::hydra::get_latest_modified_time;
use funcs::ludusavi::get_or_fetch_manifest;
use funcs::ludusavi::resolve_ludusavi_placeholders;
use funcs::ludusavi::resolve_path_pattern;
use funcs::watcher::start_process_watcher;

use crate::models::cloud::DriveFileItem;
use crate::models::cloud::DriveFileList;
use crate::models::game::GameProcessInfo;
mod funcs;
mod models;

use funcs::cloud::get_or_create_drive_folder;
use funcs::cloud::get_valid_access_token;
use funcs::hydra::copy_dir_all;
use funcs::hydra::expand_path;
use funcs::hydra::get_candidate_paths;
use funcs::hydra::get_hydra_db_dir;
use funcs::hydra::get_latest_modified_time;
use funcs::ludusavi::get_or_fetch_manifest;
use funcs::ludusavi::resolve_ludusavi_placeholders;
use funcs::ludusavi::resolve_path_pattern;
use funcs::watcher::start_process_watcher;

use crate::models::cloud::DriveFileItem;
use crate::models::cloud::DriveFileList;
use crate::models::game::GameProcessInfo;
use crate::models::game::HydraGame;
use crate::models::ludusavi::LudusaviFileRule;
use crate::models::ludusavi::LudusaviGame;
use crate::models::ludusavi::LudusaviManifest;
use crate::models::ludusavi::LudusaviSteamInfo;
use crate::models::ludusavi::LudusaviWhenCondition;
use crate::models::save::SaveInfo;
use crate::models::sync::SyncStatusResult;
use crate::models::watcher::ProcessMonitorState;

use chrono::{DateTime, Local, Utc};
use regex::Regex;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use rusty_leveldb::{LdbIterator, Options, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::SystemTime;
use sysinfo::System;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_oauth::start_with_config;
use tauri_plugin_oauth::OauthConfig;
use url::Url;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[tauri::command]
fn get_installed_games() -> Result<Vec<HydraGame>, String> {
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
fn get_game_save_info(
    app_id: Option<String>,
    title: String,
    custom_path: Option<String>,
) -> SaveInfo {
    if let Some(ref path_str) = custom_path {
        if !path_str.trim().is_empty() {
            let p = expand_path(path_str);
            if p.exists() {
                let last_mod = get_latest_modified_time(&p).map(|st| {
                    let dt: DateTime<Local> = st.into();
                    dt.format("%d/%m/%Y %H:%M").to_string()
                });

                return SaveInfo {
                    path_exists: true,
                    resolved_path: Some(p.to_string_lossy().into_owned()),
                    last_modified: last_mod,
                };
            }
        }
    }

    if let Ok(manifest) = get_or_fetch_manifest() {
        let title_lower = title.to_lowercase();

        let game_entry = manifest
            .games
            .get(&title)
            .or_else(|| {
                manifest.games.iter().find_map(|(k, v)| {
                    if k.to_lowercase().contains(&title_lower)
                        || title_lower.contains(&k.to_lowercase())
                    {
                        Some(v)
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                if let Some(ref id) = app_id {
                    if let Ok(parsed_id) = id.parse::<u64>() {
                        return manifest
                            .games
                            .values()
                            .find(|g| g.steam.as_ref().and_then(|s| s.id) == Some(parsed_id));
                    }
                }
                None
            });

        if let Some(game) = game_entry {
            if let Some(ref files) = game.files {
                for (raw_path, rule) in files {
                    let is_windows = rule.when.as_ref().map_or(true, |w| {
                        w.iter().any(|c| c.os.as_deref() == Some("windows"))
                    });

                    if is_windows {
                        let resolved_str =
                            resolve_ludusavi_placeholders(raw_path, app_id.as_deref());
                        if let Some(found_path) = resolve_path_pattern(&resolved_str) {
                            let last_mod = get_latest_modified_time(&found_path).map(|st| {
                                let dt: DateTime<Local> = st.into();
                                dt.format("%d/%m/%Y %H:%M").to_string()
                            });

                            return SaveInfo {
                                path_exists: true,
                                resolved_path: Some(found_path.to_string_lossy().into_owned()),
                                last_modified: last_mod,
                            };
                        }
                    }
                }
            }
        }
    }

    let candidates = get_candidate_paths(app_id.as_deref(), &title);
    for path in candidates {
        if path.exists() {
            let last_mod = get_latest_modified_time(&path).map(|st| {
                let dt: DateTime<Local> = st.into();
                dt.format("%d/%m/%Y %H:%M").to_string()
            });

            return SaveInfo {
                path_exists: true,
                resolved_path: Some(path.to_string_lossy().into_owned()),
                last_modified: last_mod,
            };
        }
    }

    SaveInfo {
        path_exists: false,
        resolved_path: None,
        last_modified: None,
    }
}

#[tauri::command]
async fn check_game_sync_status(
    token: String,
    game_title: String,
    save_path: String,
) -> Result<SyncStatusResult, String> {
    use chrono::Local;

    let path = Path::new(&save_path);
    if !path.exists() {
        return Err(format!(
            "Le chemin de sauvegarde local est introuvable : {}",
            save_path
        ));
    }

    let local_st = get_latest_modified_time(path)
        .ok_or("Impossible de récupérer la date de modification locale")?;
    let local_modified: DateTime<Utc> = local_st.into();

    let client = reqwest::Client::new();
    let access_token = get_valid_access_token(&client, &token).await?;
    let folder_id = get_or_create_drive_folder(&client, &token).await?;

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    let query = format!(
        "name = '{}' and mimeType = 'application/zip' and trashed = false and '{}' in parents",
        file_name, folder_id
    );

    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,modifiedTime,name,appProperties)",
        urlencoding::encode(&query)
    );

    let res = client
        .get(&url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau vérification sync : {}", e))?;

    let status = res.status();
    let err_text = res.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Erreur API Google : {}", err_text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&err_text).map_err(|e| format!("Erreur parsing JSON : {}", e))?;

    let files = json.get("files").and_then(|f| f.as_array());

    // Conversion de la date locale en heure de la machine (France / Local) pour l'affichage
    let local_in_zone = local_modified.with_timezone(&Local);
    let local_str = local_in_zone.format("%d/%m/%Y %H:%M").to_string();

    if let Some(files_array) = files {
        if let Some(file) = files_array.first() {
            // 1. On récupère la chaîne stockée dans appProperties ou le fallback
            let cloud_str_raw = if let Some(app_props) = file.get("appProperties") {
                if let Some(custom_date) = app_props.get("localModified").and_then(|v| v.as_str()) {
                    if !custom_date.is_empty() {
                        custom_date.to_string()
                    } else {
                        get_fallback_cloud_time(file)
                    }
                } else {
                    get_fallback_cloud_time(file)
                }
            } else {
                get_fallback_cloud_time(file)
            };

            // Parse de la date cloud pour la comparaison logique
            let cloud_modified_utc = DateTime::parse_from_str(
                &format!("{}:00 +0000", cloud_str_raw),
                "%d/%m/%Y %H:%M:%S %z",
            )
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(local_modified);

            // Conversion de la date cloud en heure locale pour l'affichage final
            let cloud_in_zone = cloud_modified_utc.with_timezone(&Local);
            let cloud_str = cloud_in_zone.format("%d/%m/%Y %H:%M").to_string();

            if local_modified > cloud_modified_utc {
                return Ok(SyncStatusResult {
                    status: "LocalNewer".to_string(),
                    localTime: local_str,
                    cloudTime: cloud_str,
                });
            } else if cloud_modified_utc > local_modified {
                return Ok(SyncStatusResult {
                    status: "CloudNewer".to_string(),
                    localTime: local_str,
                    cloudTime: cloud_str,
                });
            } else {
                return Ok(SyncStatusResult {
                    status: "UpToDate".to_string(),
                    localTime: local_str,
                    cloudTime: cloud_str,
                });
            }
        }
    }

    Ok(SyncStatusResult {
        status: "NotFound".to_string(),
        localTime: local_str,
        cloudTime: "Jamais".to_string(),
    })
}

fn get_fallback_cloud_time(file: &serde_json::Value) -> String {
    use chrono::Local;
    if let Some(modified_time_str) = file.get("modifiedTime").and_then(|t| t.as_str()) {
        if let Ok(cloud_modified) = DateTime::parse_from_rfc3339(modified_time_str) {
            return cloud_modified
                .with_timezone(&Local)
                .format("%d/%m/%Y %H:%M")
                .to_string();
        }
    }
    "Jamais".to_string()
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn login_google() -> Result<String, String> {
    let client_id = std::env::var("HSC-GC-ID")
        .map_err(|_| "La variable GOOGLE_CLIENT_ID est manquante".to_string())?;

    let client_secret = std::env::var("HSC-GS")
        .map_err(|_| "La variable GOOGLE_CLIENT_SECRET est manquante".to_string())?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx_cell = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    let config = OauthConfig {
        ports: Some(vec![8000, 8001, 8002]),
        response: Some("Authentification réussie ! Vous pouvez fermer cette page et retourner sur l'application.".into()),
        ..Default::default()
    };

    std::thread::spawn(move || {
        let _ = start_with_config(config, move |url| {
            if let Ok(mut sender_lock) = tx_cell.lock() {
                if let Some(sender) = sender_lock.take() {
                    let _ = sender.send(url);
                }
            }
        });
    });

    let redirect_uri = "http://localhost:8000";
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/drive.file&access_type=offline",
        client_id, redirect_uri
    );

    open::that(&auth_url).map_err(|e| e.to_string())?;

    let url_string = rx.await.map_err(|e| e.to_string())?;

    let parsed_url = Url::parse(&url_string).map_err(|e| e.to_string())?;
    let code = parsed_url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .ok_or("Code d'autorisation introuvable dans l'URL de retour".to_string())?
        .1
        .to_string();

    let client = reqwest::Client::new();
    let params = [
        ("code", code.as_str()),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    let access_token = res
        .get("access_token")
        .and_then(|t| t.as_str())
        .ok_or("Access token introuvable")?;

    let refresh_token = res
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .unwrap_or("");

    Ok(format!("{}|{}", access_token, refresh_token))
}

#[tauri::command]
async fn upload_game_save_to_drive(
    token: String,
    game_title: String,
    save_path: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let access_token = get_valid_access_token(&client, &token).await?;
    let folder_id = get_or_create_drive_folder(&client, &token).await?;

    let path = std::path::Path::new(&save_path);
    if !path.exists() {
        return Err(format!(
            "Le chemin de sauvegarde local est introuvable : {}",
            save_path
        ));
    }

    // Récupérer la date de modification locale pour l'isoler et l'envoyer en métadonnée
    let local_modified_str = get_latest_modified_time(path)
        .map(|st| {
            let dt: chrono::DateTime<chrono::Utc> = st.into();
            dt.format("%d/%m/%Y %H:%M").to_string() // Format identique à ton lastModified
        })
        .unwrap_or_else(|| "".to_string());

    let mut zip_buffer = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut zip_buffer);
        let mut zip = zip::ZipWriter::new(cursor);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if path.is_dir() {
            let walk = walkdir::WalkDir::new(path);
            for entry in walk.into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let name = entry_path.strip_prefix(path).map_err(|e| e.to_string())?;
                    zip.start_file(name.to_string_lossy(), options)
                        .map_err(|e| e.to_string())?;
                    let mut f = std::fs::File::open(entry_path).map_err(|e| e.to_string())?;
                    std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                }
            }
        } else {
            let file_name = path
                .file_name()
                .ok_or("Nom de fichier invalide")?
                .to_string_lossy();
            zip.start_file(file_name, options)
                .map_err(|e| e.to_string())?;
            let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
    }

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    // Ajout de appProperties pour stocker la vraie date locale
    let metadata = serde_json::json!({
        "name": file_name,
        "parents": [folder_id],
        "appProperties": {
            "localModified": local_modified_str
        }
    });

    let multipart = reqwest::multipart::Form::new()
        .part(
            "metadata",
            reqwest::multipart::Part::text(metadata.to_string())
                .mime_str("application/json")
                .map_err(|e| e.to_string())?,
        )
        .part(
            "file",
            reqwest::multipart::Part::bytes(zip_buffer)
                .file_name(file_name.clone())
                .mime_str("application/zip")
                .map_err(|e| e.to_string())?,
        );

    let upload_res = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .bearer_auth(&access_token)
        .multipart(multipart)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau upload : {}", e))?;

    if upload_res.status().is_success() {
        Ok(format!("Archive '{}' uploadée avec succès !", file_name))
    } else {
        let err_text = upload_res.text().await.unwrap_or_default();
        Err(format!("Erreur lors de l'upload : {}", err_text))
    }
}

#[tauri::command]
fn set_monitored_games(app: tauri::AppHandle, games: Vec<GameProcessInfo>) {
    let state = app.state::<Arc<ProcessMonitorState>>();
    let mut monitored = state.monitored_games.lock().unwrap();
    *monitored = games;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_installed_games,
            get_game_save_info,
            check_game_sync_status,
            open_folder,
            login_google,
            upload_game_save_to_drive,
            set_monitored_games
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

            Ok(())
        })
        .plugin(tauri_plugin_oauth::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("erreur lors de l'exécution de l'application tauri");
}

use crate::models::game::HydraGame;
use crate::models::ludusavi::LudusaviFileRule;
use crate::models::ludusavi::LudusaviGame;
use crate::models::ludusavi::LudusaviManifest;
use crate::models::ludusavi::LudusaviSteamInfo;
use crate::models::ludusavi::LudusaviWhenCondition;
use crate::models::save::SaveInfo;
use crate::models::sync::SyncStatusResult;
use crate::models::watcher::ProcessMonitorState;

use chrono::{DateTime, Local, Utc};
use regex::Regex;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use rusty_leveldb::{LdbIterator, Options, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::SystemTime;
use sysinfo::System;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_oauth::start_with_config;
use tauri_plugin_oauth::OauthConfig;
use url::Url;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[tauri::command]
fn get_installed_games() -> Result<Vec<HydraGame>, String> {
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
fn get_game_save_info(
    app_id: Option<String>,
    title: String,
    custom_path: Option<String>,
) -> SaveInfo {
    if let Some(ref path_str) = custom_path {
        if !path_str.trim().is_empty() {
            let p = expand_path(path_str);
            if p.exists() {
                let last_mod = get_latest_modified_time(&p).map(|st| {
                    let dt: DateTime<Local> = st.into();
                    dt.format("%d/%m/%Y %H:%M").to_string()
                });

                return SaveInfo {
                    path_exists: true,
                    resolved_path: Some(p.to_string_lossy().into_owned()),
                    last_modified: last_mod,
                };
            }
        }
    }

    if let Ok(manifest) = get_or_fetch_manifest() {
        let title_lower = title.to_lowercase();

        let game_entry = manifest
            .games
            .get(&title)
            .or_else(|| {
                manifest.games.iter().find_map(|(k, v)| {
                    if k.to_lowercase().contains(&title_lower)
                        || title_lower.contains(&k.to_lowercase())
                    {
                        Some(v)
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                if let Some(ref id) = app_id {
                    if let Ok(parsed_id) = id.parse::<u64>() {
                        return manifest
                            .games
                            .values()
                            .find(|g| g.steam.as_ref().and_then(|s| s.id) == Some(parsed_id));
                    }
                }
                None
            });

        if let Some(game) = game_entry {
            if let Some(ref files) = game.files {
                for (raw_path, rule) in files {
                    let is_windows = rule.when.as_ref().map_or(true, |w| {
                        w.iter().any(|c| c.os.as_deref() == Some("windows"))
                    });

                    if is_windows {
                        let resolved_str =
                            resolve_ludusavi_placeholders(raw_path, app_id.as_deref());
                        if let Some(found_path) = resolve_path_pattern(&resolved_str) {
                            let last_mod = get_latest_modified_time(&found_path).map(|st| {
                                let dt: DateTime<Local> = st.into();
                                dt.format("%d/%m/%Y %H:%M").to_string()
                            });

                            return SaveInfo {
                                path_exists: true,
                                resolved_path: Some(found_path.to_string_lossy().into_owned()),
                                last_modified: last_mod,
                            };
                        }
                    }
                }
            }
        }
    }

    let candidates = get_candidate_paths(app_id.as_deref(), &title);
    for path in candidates {
        if path.exists() {
            let last_mod = get_latest_modified_time(&path).map(|st| {
                let dt: DateTime<Local> = st.into();
                dt.format("%d/%m/%Y %H:%M").to_string()
            });

            return SaveInfo {
                path_exists: true,
                resolved_path: Some(path.to_string_lossy().into_owned()),
                last_modified: last_mod,
            };
        }
    }

    SaveInfo {
        path_exists: false,
        resolved_path: None,
        last_modified: None,
    }
}

#[tauri::command]
async fn check_game_sync_status(
    token: String,
    game_title: String,
    save_path: String,
) -> Result<SyncStatusResult, String> {
    use chrono::Local;

    let path = Path::new(&save_path);
    if !path.exists() {
        return Err(format!(
            "Le chemin de sauvegarde local est introuvable : {}",
            save_path
        ));
    }

    let local_st = get_latest_modified_time(path)
        .ok_or("Impossible de récupérer la date de modification locale")?;
    let local_modified: DateTime<Utc> = local_st.into();

    let client = reqwest::Client::new();
    let access_token = get_valid_access_token(&client, &token).await?;
    let folder_id = get_or_create_drive_folder(&client, &token).await?;

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    let query = format!(
        "name = '{}' and mimeType = 'application/zip' and trashed = false and '{}' in parents",
        file_name, folder_id
    );

    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,modifiedTime,name,appProperties)",
        urlencoding::encode(&query)
    );

    let res = client
        .get(&url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau vérification sync : {}", e))?;

    let status = res.status();
    let err_text = res.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Erreur API Google : {}", err_text));
    }

    let json: serde_json::Value =
        serde_json::from_str(&err_text).map_err(|e| format!("Erreur parsing JSON : {}", e))?;

    let files = json.get("files").and_then(|f| f.as_array());

    // Conversion de la date locale en heure de la machine (France / Local) pour l'affichage
    let local_in_zone = local_modified.with_timezone(&Local);
    let local_str = local_in_zone.format("%d/%m/%Y %H:%M").to_string();

    if let Some(files_array) = files {
        if let Some(file) = files_array.first() {
            // 1. On récupère la chaîne stockée dans appProperties ou le fallback
            let cloud_str_raw = if let Some(app_props) = file.get("appProperties") {
                if let Some(custom_date) = app_props.get("localModified").and_then(|v| v.as_str()) {
                    if !custom_date.is_empty() {
                        custom_date.to_string()
                    } else {
                        get_fallback_cloud_time(file)
                    }
                } else {
                    get_fallback_cloud_time(file)
                }
            } else {
                get_fallback_cloud_time(file)
            };

            // Parse de la date cloud pour la comparaison logique
            let cloud_modified_utc = DateTime::parse_from_str(
                &format!("{}:00 +0000", cloud_str_raw),
                "%d/%m/%Y %H:%M:%S %z",
            )
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(local_modified);

            // Conversion de la date cloud en heure locale pour l'affichage final
            let cloud_in_zone = cloud_modified_utc.with_timezone(&Local);
            let cloud_str = cloud_in_zone.format("%d/%m/%Y %H:%M").to_string();

            if local_modified > cloud_modified_utc {
                return Ok(SyncStatusResult {
                    status: "LocalNewer".to_string(),
                    localTime: local_str,
                    cloudTime: cloud_str,
                });
            } else if cloud_modified_utc > local_modified {
                return Ok(SyncStatusResult {
                    status: "CloudNewer".to_string(),
                    localTime: local_str,
                    cloudTime: cloud_str,
                });
            } else {
                return Ok(SyncStatusResult {
                    status: "UpToDate".to_string(),
                    localTime: local_str,
                    cloudTime: cloud_str,
                });
            }
        }
    }

    Ok(SyncStatusResult {
        status: "NotFound".to_string(),
        localTime: local_str,
        cloudTime: "Jamais".to_string(),
    })
}

fn get_fallback_cloud_time(file: &serde_json::Value) -> String {
    use chrono::Local;
    if let Some(modified_time_str) = file.get("modifiedTime").and_then(|t| t.as_str()) {
        if let Ok(cloud_modified) = DateTime::parse_from_rfc3339(modified_time_str) {
            return cloud_modified
                .with_timezone(&Local)
                .format("%d/%m/%Y %H:%M")
                .to_string();
        }
    }
    "Jamais".to_string()
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn login_google() -> Result<String, String> {
    let client_id = std::env::var("HSC-GC-ID")
        .map_err(|_| "La variable GOOGLE_CLIENT_ID est manquante".to_string())?;

    let client_secret = std::env::var("HSC-GS")
        .map_err(|_| "La variable GOOGLE_CLIENT_SECRET est manquante".to_string())?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx_cell = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    let config = OauthConfig {
        ports: Some(vec![8000, 8001, 8002]),
        response: Some("Authentification réussie ! Vous pouvez fermer cette page et retourner sur l'application.".into()),
        ..Default::default()
    };

    std::thread::spawn(move || {
        let _ = start_with_config(config, move |url| {
            if let Ok(mut sender_lock) = tx_cell.lock() {
                if let Some(sender) = sender_lock.take() {
                    let _ = sender.send(url);
                }
            }
        });
    });

    let redirect_uri = "http://localhost:8000";
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/drive.file&access_type=offline",
        client_id, redirect_uri
    );

    open::that(&auth_url).map_err(|e| e.to_string())?;

    let url_string = rx.await.map_err(|e| e.to_string())?;

    let parsed_url = Url::parse(&url_string).map_err(|e| e.to_string())?;
    let code = parsed_url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .ok_or("Code d'autorisation introuvable dans l'URL de retour".to_string())?
        .1
        .to_string();

    let client = reqwest::Client::new();
    let params = [
        ("code", code.as_str()),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    let access_token = res
        .get("access_token")
        .and_then(|t| t.as_str())
        .ok_or("Access token introuvable")?;

    let refresh_token = res
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .unwrap_or("");

    Ok(format!("{}|{}", access_token, refresh_token))
}

#[tauri::command]
async fn upload_game_save_to_drive(
    token: String,
    game_title: String,
    save_path: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let access_token = get_valid_access_token(&client, &token).await?;
    let folder_id = get_or_create_drive_folder(&client, &token).await?;

    let path = std::path::Path::new(&save_path);
    if !path.exists() {
        return Err(format!(
            "Le chemin de sauvegarde local est introuvable : {}",
            save_path
        ));
    }

    // Récupérer la date de modification locale pour l'isoler et l'envoyer en métadonnée
    let local_modified_str = get_latest_modified_time(path)
        .map(|st| {
            let dt: chrono::DateTime<chrono::Utc> = st.into();
            dt.format("%d/%m/%Y %H:%M").to_string() // Format identique à ton lastModified
        })
        .unwrap_or_else(|| "".to_string());

    let mut zip_buffer = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut zip_buffer);
        let mut zip = zip::ZipWriter::new(cursor);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if path.is_dir() {
            let walk = walkdir::WalkDir::new(path);
            for entry in walk.into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let name = entry_path.strip_prefix(path).map_err(|e| e.to_string())?;
                    zip.start_file(name.to_string_lossy(), options)
                        .map_err(|e| e.to_string())?;
                    let mut f = std::fs::File::open(entry_path).map_err(|e| e.to_string())?;
                    std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                }
            }
        } else {
            let file_name = path
                .file_name()
                .ok_or("Nom de fichier invalide")?
                .to_string_lossy();
            zip.start_file(file_name, options)
                .map_err(|e| e.to_string())?;
            let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
    }

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    // Ajout de appProperties pour stocker la vraie date locale
    let metadata = serde_json::json!({
        "name": file_name,
        "parents": [folder_id],
        "appProperties": {
            "localModified": local_modified_str
        }
    });

    let multipart = reqwest::multipart::Form::new()
        .part(
            "metadata",
            reqwest::multipart::Part::text(metadata.to_string())
                .mime_str("application/json")
                .map_err(|e| e.to_string())?,
        )
        .part(
            "file",
            reqwest::multipart::Part::bytes(zip_buffer)
                .file_name(file_name.clone())
                .mime_str("application/zip")
                .map_err(|e| e.to_string())?,
        );

    let upload_res = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .bearer_auth(&access_token)
        .multipart(multipart)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau upload : {}", e))?;

    if upload_res.status().is_success() {
        Ok(format!("Archive '{}' uploadée avec succès !", file_name))
    } else {
        let err_text = upload_res.text().await.unwrap_or_default();
        Err(format!("Erreur lors de l'upload : {}", err_text))
    }
}

#[tauri::command]
fn set_monitored_games(app: tauri::AppHandle, games: Vec<GameProcessInfo>) {
    let state = app.state::<Arc<ProcessMonitorState>>();
    let mut monitored = state.monitored_games.lock().unwrap();
    *monitored = games;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_installed_games,
            get_game_save_info,
            check_game_sync_status,
            open_folder,
            login_google,
            upload_game_save_to_drive,
            set_monitored_games
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

            Ok(())
        })
        .plugin(tauri_plugin_oauth::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("erreur lors de l'exécution de l'application tauri");
}
