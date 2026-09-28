use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

use tauri_plugin_oauth::start_with_config;
use tauri_plugin_oauth::OauthConfig;

use chrono::{DateTime, Local};
use regex::Regex;
use rusty_leveldb::{LdbIterator, Options, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use url::Url;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

const MANIFEST_URL: &str = "https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.json";

// --- Structures Hydra ---

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HydraGame {
    pub title: String,
    pub object_id: String,
    pub shop: Option<String>,
    pub executable_path: Option<String>,
    pub is_deleted: Option<bool>,
    pub icon_url: Option<String>,
    pub has_active_steam_import: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveInfo {
    pub path_exists: bool,
    pub resolved_path: Option<String>,
    pub last_modified: Option<String>,
}

// --- Structures du Manifest Ludusavi ---

#[derive(Debug, Deserialize)]
pub struct LudusaviManifest {
    pub games: HashMap<String, LudusaviGame>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviGame {
    pub files: Option<HashMap<String, LudusaviFileRule>>,
    pub steam: Option<LudusaviSteamInfo>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviFileRule {
    pub when: Option<Vec<LudusaviWhenCondition>>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviWhenCondition {
    pub os: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviSteamInfo {
    pub id: Option<u64>,
}

// --- Gestion du téléchargement du Manifest ---

pub fn get_or_fetch_manifest() -> Result<LudusaviManifest, String> {
    let cache_dir = env::temp_dir().join("hydra_companion");
    let manifest_path = cache_dir.join("ludusavi_manifest.json");

    let _ = fs::create_dir_all(&cache_dir);

    let should_download = if !manifest_path.exists() {
        true
    } else if let Ok(metadata) = fs::metadata(&manifest_path) {
        if let Ok(modified) = metadata.modified() {
            modified.elapsed().map(|d| d.as_secs() > 7 * 86400).unwrap_or(true)
        } else {
            true
        }
    } else {
        true
    };

    if should_download {
        if let Ok(response) = reqwest::blocking::get(MANIFEST_URL) {
            if let Ok(bytes) = response.bytes() {
                let _ = fs::write(&manifest_path, bytes);
            }
        }
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Impossible de lire le manifest : {}", e))?;

    serde_json::from_str::<LudusaviManifest>(&content)
        .map_err(|e| format!("Erreur de parsing du manifest Ludusavi : {}", e))
}

// --- Mappeur de variables Ludusavi ---

#[cfg(target_os = "windows")]
fn get_active_steam_user_id() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let steam_key = hkcu.open_subkey(r"Software\Valve\Steam\ActiveProcess").ok()?;
    let user_id: u32 = steam_key.get_value("ActiveUser").ok()?;
    if user_id != 0 {
        Some(user_id.to_string())
    } else {
        None
    }
}

fn resolve_ludusavi_placeholders(path_str: &str, app_id: Option<&str>) -> String {
    let mut resolved = path_str.to_string();

    #[cfg(target_os = "windows")]
    {
        let user_profile = env::var("USERPROFILE").unwrap_or_default();
        let appdata = env::var("APPDATA").unwrap_or_default();
        let localappdata = env::var("LOCALAPPDATA").unwrap_or_default();
        let public = env::var("PUBLIC").unwrap_or_else(|_| r"C:\Users\Public".to_string());

        resolved = resolved.replace("<winDocuments>", &format!(r"{}\Documents", user_profile));
        resolved = resolved.replace("<winAppData>", &appdata);
        resolved = resolved.replace("<winLocalAppData>", &localappdata);
        resolved = resolved.replace("<winLocalAppDataLow>", &format!(r"{}\AppData\LocalLow", user_profile));
        resolved = resolved.replace("<winSavedGames>", &format!(r"{}\Saved Games", user_profile));
        resolved = resolved.replace("<winPublic>", &public);
        resolved = resolved.replace("<winProgramData>", r"C:\ProgramData");

        if resolved.contains("<storeUserId>") || resolved.contains("<steamUser>") {
            let active_steam_id = get_active_steam_user_id().unwrap_or_else(|| "*".to_string());
            resolved = resolved.replace("<storeUserId>", &active_steam_id);
            resolved = resolved.replace("<steamUser>", &active_steam_id);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = env::var("HOME").unwrap_or_default();
        resolved = resolved.replace("<home>", &home);
        resolved = resolved.replace("<xdgConfig>", &format!("{}/.config", home));
        resolved = resolved.replace("<xdgData>", &format!("{}/.local/share", home));
    }

    if let Some(id) = app_id {
        resolved = resolved.replace("<game>", id);
    }

    resolved.replace('/', "\\")
}

fn resolve_path_pattern(pattern: &str) -> Option<PathBuf> {
    let path = PathBuf::from(pattern);

    if path.exists() {
        return Some(path);
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('\\').collect();
        let mut current_base = PathBuf::from(parts[0]);

        for part in &parts[1..] {
            if part.contains('*') {
                if let Ok(entries) = fs::read_dir(&current_base) {
                    let mut found = false;
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            current_base = entry.path();
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                current_base = current_base.join(part);
            }
        }

        if current_base.exists() {
            return Some(current_base);
        }
    }

    None
}

// --- Helpers de système de fichiers ---

fn get_hydra_db_dir() -> Result<PathBuf, String> {
    let base_dir = if cfg!(target_os = "windows") {
        let appdata = env::var("APPDATA").map_err(|_| "Variable APPDATA introuvable")?;
        PathBuf::from(appdata).join("hydralauncher")
    } else {
        let home = env::var("HOME").map_err(|_| "Variable HOME introuvable")?;
        let config_path = PathBuf::from(&home).join(".config/hydralauncher");
        if config_path.exists() {
            config_path
        } else {
            PathBuf::from(home).join(".local/share/hydralauncher")
        }
    };

    Ok(base_dir.join("hydra-db"))
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name == "LOCK" {
            continue;
        }
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(file_name))?;
        } else {
            let _ = fs::copy(entry.path(), dst.join(file_name));
        }
    }
    Ok(())
}

fn expand_path(path_str: &str) -> PathBuf {
    let mut expanded = path_str.to_string();

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            expanded = expanded.replace("%APPDATA%", &appdata);
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            expanded = expanded.replace("%LOCALAPPDATA%", &localappdata);
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            expanded = expanded.replace("%USERPROFILE%", &userprofile);
        }
        if let Ok(public) = std::env::var("PUBLIC") {
            expanded = expanded.replace("%PUBLIC%", &public);
        } else {
            expanded = expanded.replace("%PUBLIC%", r"C:\Users\Public");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            if expanded.starts_with('~') {
                expanded = expanded.replacen('~', &home, 1);
            }
            expanded = expanded.replace("$HOME", &home);
        }
    }

    PathBuf::from(expanded)
}

fn get_latest_modified_time(dir: &Path) -> Option<SystemTime> {
    if !dir.exists() {
        return None;
    }

    if dir.is_file() {
        return dir.metadata().ok()?.modified().ok();
    }

    let mut latest: Option<SystemTime> = None;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let time = if path.is_dir() {
                get_latest_modified_time(&path)
            } else {
                path.metadata().ok()?.modified().ok()
            };

            if let Some(t) = time {
                if latest.is_none() || t > latest.unwrap() {
                    latest = Some(t);
                }
            }
        }
    }

    latest
}

fn get_candidate_paths(app_id: Option<&str>, title: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let clean_title = title.replace(':', "").replace('/', "").replace('\\', "");

    // 1. Si un AppID est transmis
    if let Some(id) = app_id {
        if !id.trim().is_empty() {
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\RUNE\{}", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\RUNE\{}\remote", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\CODEX\{}", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\CODEX\{}\remote", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\TENOKE\{}", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\EMPRESS\{}\remote", id)));

            candidates.push(expand_path(&format!(r"%APPDATA%\Goldberg SteamEmu Saves\{}", id)));
            candidates.push(expand_path(&format!(r"%APPDATA%\Goldberg SteamEmu Saves\{}\remote", id)));
            candidates.push(expand_path(&format!(r"%APPDATA%\FLT\{}", id)));
            candidates.push(expand_path(&format!(r"%APPDATA%\Razor1911Appdata\{}", id)));

            candidates.push(expand_path(&format!(r"C:\Program Files (x86)\Steam\userdata\*\*\remote")));
        }
    }

    // 2. Fallbacks spécifiques pour The Witcher 3 (AppIDs connus + dossier GOG)
    if title.to_lowercase().contains("witcher") {
        let tw3_ids = ["292030", "378120", "499450"];
        for id in tw3_ids {
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\RUNE\{}\remote", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\RUNE\{}", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\CODEX\{}\remote", id)));
            candidates.push(expand_path(&format!(r"%PUBLIC%\Documents\Steam\CODEX\{}", id)));
            candidates.push(expand_path(&format!(r"%APPDATA%\Goldberg SteamEmu Saves\{}\remote", id)));
            candidates.push(expand_path(&format!(r"%APPDATA%\Goldberg SteamEmu Saves\{}", id)));
        }

        candidates.push(expand_path(r"%USERPROFILE%\Documents\The Witcher 3\gamesaves"));
        candidates.push(expand_path(r"%USERPROFILE%\Documents\The Witcher 3 Wild Hunt\gamesaves"));
    }

    // 3. Chemins génériques par nom de dossier
    candidates.push(expand_path(&format!(r"%USERPROFILE%\Saved Games\{}", clean_title)));
    candidates.push(expand_path(&format!(r"%USERPROFILE%\Documents\{}\gamesaves", clean_title)));
    candidates.push(expand_path(&format!(r"%USERPROFILE%\Documents\{}", clean_title)));
    candidates.push(expand_path(&format!(r"%LOCALAPPDATA%\{}\Saved\SaveGames", clean_title)));
    candidates.push(expand_path(&format!(r"%LOCALAPPDATA%\{}", clean_title)));
    candidates.push(expand_path(&format!(r"%APPDATA%\{}", clean_title)));

    candidates
}

// --- Commandes Tauri ---

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
    let mut iter = db.new_iter().map_err(|e| format!("Erreur d'itérateur : {:?}", e))?;

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
    // 1. Chemin personnalisé
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

    // 2. Recherche via Manifest Ludusavi
    if let Ok(manifest) = get_or_fetch_manifest() {
        let title_lower = title.to_lowercase();

        // Recherche souple : Titre exact, contient le titre, ou via AppID Steam
        let game_entry = manifest.games.get(&title)
            .or_else(|| {
                manifest.games.iter().find_map(|(k, v)| {
                    if k.to_lowercase().contains(&title_lower) || title_lower.contains(&k.to_lowercase()) {
                        Some(v)
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                if let Some(ref id) = app_id {
                    if let Ok(parsed_id) = id.parse::<u64>() {
                        return manifest.games.values().find(|g| {
                            g.steam.as_ref().and_then(|s| s.id) == Some(parsed_id)
                        });
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
                        let resolved_str = resolve_ludusavi_placeholders(raw_path, app_id.as_deref());
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

    // 3. Fallback sur les candidats manuels
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
    // Tu mets tes vraies clés directement ici (ou via env!("GOOGLE_CLIENT_ID"))
    let client_id = "742327849744-gsham3lda4i5pm37jmu2cj10c6mf215i.apps.googleusercontent.com";
    let client_secret = "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";

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
        ("client_id", client_id),
        ("client_secret", client_secret),
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

    let access_token = res.get("access_token")
        .and_then(|t| t.as_str())
        .ok_or("Access token introuvable")?;
        
    // Récupère aussi le refresh token si Google le renvoie (présent à la première connexion)
    let refresh_token = res.get("refresh_token")
        .and_then(|t| t.as_str())
        .unwrap_or("");

    // Tu peux renvoyer un JSON ou une structure combinée, par exemple "access_token|refresh_token"
    Ok(format!("{}|{}", access_token, refresh_token))
}

#[tauri::command]
async fn upload_game_save_to_drive(
    token: String,
    game_title: String,
    save_content: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    // Étape 1 : Chercher le dossier "hydra-save-companion"
    let query = "name = 'hydra-save-companion' and mimeType = 'application/vnd.google-apps.folder' and trashed = false";
    let search_res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .query(&[("q", query)])
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau recherche : {}", e))?;

    if !search_res.status().is_success() {
        let err_body = search_res.text().await.unwrap_or_default();
        return Err(format!("Erreur API Google (Recherche) : {}", err_body));
    }

    let search_json = search_res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Erreur parsing JSON recherche : {}", e))?;

    let folder_id = if let Some(files) = search_json.get("files").and_then(|f| f.as_array()) {
        if let Some(folder) = files.first() {
            folder.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string()
        } else {
            // Le dossier n'existe pas, on le crée
            let folder_metadata = serde_json::json!({
                "name": "hydra-save-companion",
                "mimeType": "application/vnd.google-apps.folder"
            });

            let create_res = client
                .post("https://www.googleapis.com/drive/v3/files")
                .bearer_auth(&token)
                .json(&folder_metadata)
                .send()
                .await
                .map_err(|e| format!("Erreur réseau création dossier : {}", e))?;

            if !create_res.status().is_success() {
                let err_body = create_res.text().await.unwrap_or_default();
                return Err(format!("Erreur API Google (Création dossier) : {}", err_body));
            }

            let create_json = create_res
                .json::<serde_json::Value>()
                .await
                .map_err(|e| format!("Erreur parsing JSON création : {}", e))?;

            create_json.get("id")
                .and_then(|i| i.as_str())
                .ok_or("ID du dossier introuvable après création".to_string())?
                .to_string()
        }
    } else {
        return Err("Format de réponse Google Drive invalide".to_string());
    };

    // Étape 2 : Upload du fichier .txt
    let file_name = format!("{}.txt", game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_"));
    
    let metadata = serde_json::json!({
        "name": file_name,
        "parents": [folder_id]
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
            reqwest::multipart::Part::text(save_content)
                .mime_str("text/plain")
                .map_err(|e| e.to_string())?,
        );

    let upload_res = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .bearer_auth(&token)
        .multipart(multipart)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau upload : {}", e))?;

    if upload_res.status().is_success() {
        Ok(format!("Fichier '{}' uploadé avec succès !", file_name))
    } else {
        let err_text = upload_res.text().await.unwrap_or_default();
        Err(format!("Erreur lors de l'upload : {}", err_text))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_installed_games,
            get_game_save_info,
            open_folder,
            login_google,
            upload_game_save_to_drive
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

            Ok(())
        })
        .plugin(tauri_plugin_oauth::init())
        .run(tauri::generate_context!())
        .expect("erreur lors de l'exécution de l'application tauri");
}