use chrono::{DateTime, Local};
use std::fs;
use std::path::Path;

use crate::funcs::cloud::get_or_create_drive_folder;
use crate::funcs::cloud::get_valid_access_token;
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
) -> Result<SyncStatusResult, String> {
    use chrono::{DateTime, Local, Timelike, Utc};

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

    if let Some(files_array) = files {
        if let Some(file) = files_array.first() {
            let cloud_modified_utc = file
                .get("modifiedTime")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| {
                    let utc = dt.with_timezone(&Utc);
                    utc.with_second(0).unwrap().with_nanosecond(0).unwrap()
                })
                .unwrap_or_else(|| Utc::now());

            let cloud_in_zone = cloud_modified_utc.with_timezone(&Local);
            let cloud_str = cloud_in_zone.format("%d/%m/%Y %H:%M").to_string();

            let (local_utc, local_str) = match local_modified_info {
                Some(info) => info,
                None => {
                    return Ok(SyncStatusResult {
                        status: "CloudNewer".to_string(),
                        localTime: "Aucune".to_string(),
                        cloudTime: cloud_str,
                    });
                }
            };

            if local_utc > cloud_modified_utc {
                return Ok(SyncStatusResult {
                    status: "LocalNewer".to_string(),
                    localTime: local_str,
                    cloudTime: cloud_str,
                });
            } else if cloud_modified_utc > local_utc {
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

    let local_str = local_modified_info
        .map(|(_, str_val)| str_val)
        .unwrap_or_else(|| "Jamais".to_string());

    Ok(SyncStatusResult {
        status: "NotFound".to_string(),
        localTime: local_str,
        cloudTime: "Jamais".to_string(),
    })
}

use reqwest;
use serde_json::Value;

#[tauri::command]
pub async fn get_steam_cover(app_id: Option<i64>) -> Result<String, String> {
    println!(
        "\n[DEBUG COVER] Récupération cover pour AppID: {:?}",
        app_id
    );

    let default_cover = "/path/to/default-placeholder.jpg".to_string();

    let id = match app_id {
        Some(val) if val > 0 => val,
        _ => {
            println!("[DEBUG COVER] -> AppID invalide ou absent, utilisation du placeholder.");
            return Ok(default_cover);
        }
    };

    let url = format!(
        "https://store.steampowered.com/api/appdetails?appids={}",
        id
    );

    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(res) => {
            println!("[DEBUG COVER] -> Requête HTTP réussie pour l'ID {}", id);
            res
        }
        Err(e) => {
            println!(
                "[DEBUG COVER] -> Erreur réseau HTTP: {}, fallback CDN direct",
                e
            );
            return Ok(format!(
                "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/library_600x900_2x.jpg",
                id
            ));
        }
    };

    let text_resp = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            println!("[DEBUG COVER] -> Erreur lecture texte réponse: {}", e);
            String::new()
        }
    };

    println!(
        "[DEBUG COVER] -> Réponse brute API Steam (premiers 300 caractères) : {}",
        &text_resp.chars().take(300).collect::<String>()
    );

    let json: Value = match serde_json::from_str(&text_resp) {
        Ok(j) => j,
        Err(e) => {
            println!("[DEBUG COVER] -> Erreur parsing JSON: {}", e);
            return Ok(format!(
                "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/library_600x900_2x.jpg",
                id
            ));
        }
    };

    if let Some(app_data) = json.get(&id.to_string()) {
        let success = app_data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        println!(
            "[DEBUG COVER] -> Champ 'success' pour l'ID {} : {}",
            id, success
        );

        if success {
            if let Some(data) = app_data.get("data") {
                if let Some(header) = data.get("header_image").and_then(|v| v.as_str()) {
                    println!("[DEBUG COVER] -> Trouvé header_image : {}", header);
                    return Ok(header.to_string());
                }
                if let Some(capsule) = data.get("capsule_image").and_then(|v| v.as_str()) {
                    println!("[DEBUG COVER] -> Trouvé capsule_image : {}", capsule);
                    return Ok(capsule.to_string());
                }
                println!("[DEBUG COVER] -> 'data' présent mais ni header_image ni capsule_image trouvés.");
            } else {
                println!("[DEBUG COVER] -> Objet 'data' absent dans le JSON.");
            }
        } else {
            println!("[DEBUG COVER] -> L'API Steam indique success: false (jeu retiré, ID invalide ou région bloquée).");
        }
    } else {
        println!(
            "[DEBUG COVER] -> Clé '{}' absente à la racine du JSON Steam.",
            id
        );
    }

    let fallback = format!(
        "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/library_600x900_2x.jpg",
        id
    );
    println!("[DEBUG COVER] -> Fallback final utilisé : {}", fallback);
    Ok(fallback)
}
