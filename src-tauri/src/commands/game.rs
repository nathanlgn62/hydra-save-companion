use chrono::{DateTime, Local};
use std::path::Path;

use crate::funcs::cloud::check_sync_dropbox;
use crate::funcs::cloud::check_sync_google_drive;
use crate::funcs::cloud::check_sync_proton_drive;
use crate::funcs::hydra::expand_path;
use crate::funcs::hydra::get_latest_modified_time;
use crate::funcs::ludusavi::get_cached_manifest;
use crate::funcs::ludusavi::resolve_ludusavi_placeholders;
use crate::funcs::ludusavi::resolve_path_pattern;
use crate::models::save::SaveInfo;
use crate::models::sync::SyncStatusResult;

#[tauri::command]
pub fn get_game_save_info(
    app_id: Option<String>,
    title: String,
    custom_path: Option<String>,
) -> SaveInfo {
    println!(
        "\n[DEBUG] Recherche save pour le jeu: '{}' (AppID: {:?})",
        title, app_id
    );

    if let Some(ref path_str) = custom_path {
        if !path_str.trim().is_empty() {
            let p = expand_path(path_str);
            let exists = p.exists();
            if exists {
                println!("[DEBUG] -> Trouvé via custom_path : {:?}", p);
                let last_mod = get_latest_modified_time(&p).map(|st| {
                    let dt: DateTime<Local> = st.into();
                    dt.format("%d/%m/%Y %H:%M").to_string()
                });

                return SaveInfo {
                    ludasavi_path_exists: true,
                    local_path_exists: true,
                    resolved_path: Some(p.to_string_lossy().into_owned()),
                    last_modified: last_mod,
                };
            }
        }
    }

    // Vérification du dossier de sauvegarde de démonstration/simulation
    let demo_folder = std::env::temp_dir()
        .join("hydra_companion_demo_saves")
        .join(&title);
    if demo_folder.exists() {
        println!("[DEBUG] -> Trouvé via demo saves : {:?}", demo_folder);
        let last_mod = get_latest_modified_time(&demo_folder).map(|st| {
            let dt: DateTime<Local> = st.into();
            dt.format("%d/%m/%Y %H:%M").to_string()
        });

        return SaveInfo {
            ludasavi_path_exists: true,
            local_path_exists: true,
            resolved_path: Some(demo_folder.to_string_lossy().into_owned()),
            last_modified: last_mod,
        };
    }

    match get_cached_manifest() {
        Ok(manifest) => {
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
                println!("[DEBUG] Jeu trouvé dans le manifeste Ludasavi !");
                if let Some(ref files) = game.files {
                    let mut best_resolved_str: Option<String> = None;

                    for (raw_path, rule) in files {
                        let is_windows = rule.when.as_ref().map_or(true, |w| {
                            w.iter().any(|c| c.os.as_deref() == Some("windows"))
                        });

                        if is_windows {
                            let resolved_str =
                                resolve_ludusavi_placeholders(raw_path, app_id.as_deref());

                            let lower = resolved_str.to_lowercase();

                            if lower.ends_with(".png")
                                || lower.ends_with(".jpg")
                                || lower.ends_with(".txt")
                                || lower.ends_with(".log")
                                || lower.ends_with(".settings")
                                || lower.ends_with(".ini")
                            {
                                if best_resolved_str.is_none() {
                                    best_resolved_str = Some(resolved_str);
                                }
                                continue;
                            }

                            if lower.contains("save")
                                || lower.contains("saved")
                                || lower.contains("profile")
                            {
                                best_resolved_str = Some(resolved_str);
                                break;
                            }

                            if best_resolved_str.is_none() {
                                best_resolved_str = Some(resolved_str);
                            }
                        }
                    }

                    if let Some(resolved_str) = best_resolved_str {
                        println!("[DEBUG] Manifest - Chemin retenu : {}", resolved_str);

                        // Si le chemin contient un joker (*), on essaie de trouver le fichier correspondant
                        // ou on se replie sur le dossier parent pour l'existence et la date
                        let evaluated_path = if resolved_str.contains('*') {
                            if let Some(parent) = Path::new(&resolved_str).parent() {
                                if parent.exists() {
                                    // On prend le dossier parent (ex: gamesaves) pour que .exists() et les stats fonctionnent
                                    parent.to_path_buf()
                                } else {
                                    resolve_path_pattern(&resolved_str)
                                        .unwrap_or_else(|| expand_path(&resolved_str))
                                }
                            } else {
                                resolve_path_pattern(&resolved_str)
                                    .unwrap_or_else(|| expand_path(&resolved_str))
                            }
                        } else {
                            resolve_path_pattern(&resolved_str)
                                .unwrap_or_else(|| expand_path(&resolved_str))
                        };

                        let exists = evaluated_path.exists();
                        let path_str = evaluated_path.to_string_lossy().into_owned();

                        let last_mod = if exists {
                            get_latest_modified_time(&evaluated_path).map(|st| {
                                let dt: DateTime<Local> = st.into();
                                dt.format("%d/%m/%Y %H:%M").to_string()
                            })
                        } else {
                            None
                        };

                        return SaveInfo {
                            ludasavi_path_exists: true,
                            local_path_exists: exists,
                            resolved_path: Some(path_str),
                            last_modified: last_mod,
                        };
                    }
                } else {
                    println!("[DEBUG] Le jeu trouvé dans le manifeste n'a pas de section 'files' définie.");
                }
            } else {
                println!("[DEBUG] Jeu introuvable dans le manifeste Ludasavi.");
            }
        }
        Err(e) => {
            println!("[DEBUG] Erreur chargement manifest Ludasavi: {}", e);
        }
    }

    println!("[DEBUG] Aucun chemin valide trouvé via le manifeste.");
    SaveInfo {
        ludasavi_path_exists: false,
        local_path_exists: false,
        resolved_path: None,
        last_modified: None,
    }
}

#[tauri::command]
pub async fn check_game_sync_status(
    token: String,
    game_title: String,
    save_path: String,
    provider: Option<String>,
) -> Result<SyncStatusResult, String> {
    use chrono::{DateTime, Local, Timelike, Utc};

    let provider_name = provider.as_deref().unwrap_or("google-drive").to_lowercase();

    let path = Path::new(&save_path);

    let local_modified_info: Option<(DateTime<Utc>, String)> = if path.exists() {
        get_latest_modified_time(path).map(|st| {
            let utc: DateTime<Utc> = st.into();
            let utc_truncated = utc.with_second(0).unwrap().with_nanosecond(0).unwrap();
            let local_str = utc_truncated
                .with_timezone(&Local)
                .format("%d/%m/%Y %H:%M")
                .to_string();
            (utc_truncated, local_str)
        })
    } else {
        None
    };

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    match provider_name.as_str() {
        "google-drive" | "gdrive" | "google" => {
            check_sync_google_drive(token, file_name, local_modified_info).await
        }
        "dropbox" => check_sync_dropbox(token, file_name, local_modified_info).await,
        "proton-drive" | "proton" => check_sync_proton_drive(file_name, local_modified_info).await,
        other => Err(format!("Provider '{}' non supporté", other)),
    }
}

async fn get_steamgriddb_cover(
    client: &reqwest::Client,
    app_id: Option<i64>,
    title: Option<&str>,
) -> Option<String> {
    let api_key = std::env::var("STEAM_GRID_DB").ok()?;
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return None;
    }

    let extract_grid_url = |json: &serde_json::Value| -> Option<String> {
        let data = json.get("data")?.as_array()?;
        let first_grid = data.first()?;
        first_grid
            .get("url")
            .or_else(|| first_grid.get("thumb"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
    };

    // 1. Recherche par app_id Steam sur SteamGridDB
    if let Some(id) = app_id {
        if id > 0 {
            let url_600x900 = format!(
                "https://www.steamgriddb.com/api/v2/grids/steam/{}?dimensions=600x900",
                id
            );
            if let Ok(resp) = client.get(&url_600x900).bearer_auth(api_key).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(cover_url) = extract_grid_url(&json) {
                        return Some(cover_url);
                    }
                }
            }

            // Fallback sans restriction de dimension si pas de 600x900 strict
            let url_any = format!("https://www.steamgriddb.com/api/v2/grids/steam/{}", id);
            if let Ok(resp) = client.get(&url_any).bearer_auth(api_key).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(cover_url) = extract_grid_url(&json) {
                        return Some(cover_url);
                    }
                }
            }
        }
    }

    // 2. Recherche par titre du jeu sur SteamGridDB
    if let Some(game_title) = title {
        let clean_title = game_title
            .replace(['™', '®', '©'], "")
            .trim()
            .to_string();

        if !clean_title.is_empty() {
            let encoded_title = urlencoding::encode(&clean_title);
            let search_url = format!(
                "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
                encoded_title
            );

            if let Ok(resp) = client.get(&search_url).bearer_auth(api_key).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                        if let Some(first_game) = data.first() {
                            if let Some(sgdb_id) = first_game.get("id").and_then(|i| i.as_i64()) {
                                let grid_url_600x900 = format!(
                                    "https://www.steamgriddb.com/api/v2/grids/game/{}?dimensions=600x900",
                                    sgdb_id
                                );
                                if let Ok(grid_resp) = client.get(&grid_url_600x900).bearer_auth(api_key).send().await {
                                    if let Ok(grid_json) = grid_resp.json::<serde_json::Value>().await {
                                        if let Some(cover_url) = extract_grid_url(&grid_json) {
                                            return Some(cover_url);
                                        }
                                    }
                                }

                                let grid_url_any = format!(
                                    "https://www.steamgriddb.com/api/v2/grids/game/{}",
                                    sgdb_id
                                );
                                if let Ok(grid_resp) = client.get(&grid_url_any).bearer_auth(api_key).send().await {
                                    if let Ok(grid_json) = grid_resp.json::<serde_json::Value>().await {
                                        if let Some(cover_url) = extract_grid_url(&grid_json) {
                                            return Some(cover_url);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[tauri::command]
pub async fn get_steam_cover(
    app_id: Option<i64>,
    title: Option<String>,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    // On passe directement et uniquement par SteamGridDB (clé STEAM_GRID_DB via Infisical)
    if let Some(sgdb_cover) = get_steamgriddb_cover(&client, app_id, title.as_deref()).await {
        return Ok(sgdb_cover);
    }

    // Fallback chaîne vide (le frontend gère le cas image error / absence de jaquette)
    Ok(String::new())
}
