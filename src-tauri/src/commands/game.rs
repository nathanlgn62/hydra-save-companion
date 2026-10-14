use chrono::{DateTime, Local};
use std::path::Path;

use crate::funcs::cloud::check_sync_dropbox;
use crate::funcs::cloud::check_sync_google_drive;
use crate::funcs::cloud::check_sync_proton_drive;
use crate::funcs::hydra::expand_path;
use crate::funcs::hydra::get_latest_modified_time;
use crate::funcs::ludasavi::find_game_in_manifest;
use crate::funcs::ludasavi::resolve_ludasavi_placeholders;
use crate::funcs::ludasavi::resolve_path_pattern;
use crate::funcs::steam::get_steamgriddb_cover;
use crate::models::save::SaveInfo;
use crate::models::sync::SyncStatusResult;

#[tauri::command]
pub async fn get_game_save_info(
    app_id: Option<String>,
    title: String,
    custom_path: Option<String>,
    executable_path: Option<String>,
) -> SaveInfo {
    tokio::task::spawn_blocking(move || {
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

    if let Some(game) = find_game_in_manifest(&title, app_id.as_deref()) {
        println!("[DEBUG] Jeu trouvé dans le manifeste Ludasavi !");
        if let Some(ref files) = game.files {
            let mut best_resolved_str: Option<String> = None;

                    for (raw_path, rule) in files {
                        let is_windows = rule.when.as_ref().is_none_or(|w| {
                            w.iter().any(|c| c.os.as_deref() == Some("windows"))
                        });

                        if is_windows {
                            let resolved_str = resolve_ludasavi_placeholders(
                                raw_path,
                                app_id.as_deref(),
                                executable_path.as_deref(),
                            );

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

                        let evaluated_path = resolve_path_pattern(&resolved_str).unwrap_or_else(|| {
                            if resolved_str.contains('*') {
                                Path::new(&resolved_str)
                                    .parent()
                                    .filter(|p| p.exists())
                                    .map(|p| p.to_path_buf())
                                    .unwrap_or_else(|| expand_path(&resolved_str))
                            } else {
                                expand_path(&resolved_str)
                            }
                        });

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

        println!("[DEBUG] Aucun chemin valide trouvé via le manifeste.");
        SaveInfo {
            ludasavi_path_exists: false,
            local_path_exists: false,
            resolved_path: None,
            last_modified: None,
        }
    })
    .await
    .unwrap_or(SaveInfo {
        ludasavi_path_exists: false,
        local_path_exists: false,
        resolved_path: None,
        last_modified: None,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_save_info_4_games() {
        let games = vec![
            (
                "STAR WARS Zero Company™",
                Some("2075800".to_string()),
                Some("D:\\Games\\STAR WARS Zero Company\\SWZeroCompany.exe".to_string()),
            ),
            (
                "The Witcher 3: Wild Hunt",
                Some("292030".to_string()),
                Some("C:\\Games\\The Witcher 3 Wild Hunt GOTY\\bin\\x64_dx12\\witcher3.exe".to_string()),
            ),
            (
                "METAL GEAR SOLID 4: Guns of the Patriots - Master Collection Version",
                Some("2492670".to_string()),
                Some("D:\\Games\\METAL GEAR SOLID 4 Guns of the Patriots Master Collection Version\\Launcher\\launcher.exe".to_string()),
            ),
            (
                "1666: Amsterdam",
                Some("3949550".to_string()),
                Some("D:\\Games\\1666 Amsterdam\\1666Amsterdam\\Amsterdam\\Binaries\\Win64\\1666Amsterdam.exe".to_string()),
            ),
            (
                "METAL GEAR SOLID: Peace Walker - Master Collection Version",
                Some("4184360".to_string()),
                Some("D:\\Games\\METAL GEAR SOLID Peace Walker Master Collection Version\\launcher\\launcher.exe".to_string()),
            ),
        ];

        for (title, app_id, exec) in games {
            let res = get_game_save_info(app_id.clone(), title.to_string(), None, exec).await;
            println!("\n=== RESULT FOR '{}' (app_id: {:?}) ===", title, app_id);
            println!("  ludasavi_path_exists: {}", res.ludasavi_path_exists);
            println!("  local_path_exists: {}", res.local_path_exists);
            println!("  resolved_path: {:?}", res.resolved_path);
            println!("  last_modified: {:?}", res.last_modified);
            assert!(res.ludasavi_path_exists, "Game '{}' should be found in Ludasavi manifest!", title);
        }
    }
}

