use crate::models::sync::{RemoteBackupInfo, SyncStatusResult};
use crate::utils::utils::compare_and_build_result;
use reqwest;
use std::fs;

pub async fn get_valid_access_token(
    client: &reqwest::Client,
    token_store: &str,
    provider: Option<&str>,
) -> Result<String, String> {
    let parts: Vec<&str> = token_store.split('|').collect();
    let access_token = parts.first().unwrap_or(&"");
    let refresh_token = parts.get(1).unwrap_or(&"");

    if refresh_token.is_empty() {
        return Ok(access_token.to_string());
    }

    let p = provider.unwrap_or("google-drive").to_lowercase();
    let (client_id, client_secret, token_url) = match p.as_str() {
        "google-drive" | "gdrive" | "google" => {
            let id = std::env::var("HSC_GC_ID").unwrap_or_else(|_| {
                "742327849744-gsham3lda4i5pm37jmu2cj10c6mf215i.apps.googleusercontent.com"
                    .to_string()
            });
            let secret = std::env::var("HSC_GS")
                .unwrap_or_else(|_| "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".to_string());
            (id, secret, "https://oauth2.googleapis.com/token")
        }
        "dropbox" => {
            let id = std::env::var("HSC_DBX_ID").unwrap_or_default();
            let secret = std::env::var("HSC_DBX_SECRET").unwrap_or_default();
            (id, secret, "https://api.dropbox.com/oauth2/token")
        }
        "proton-drive" | "proton" => {
            let id = std::env::var("HSC_PROTON_ID").unwrap_or_default();
            let secret = std::env::var("HSC_PROTON_SECRET").unwrap_or_default();
            (id, secret, "https://account.proton.me/oauth/token")
        }
        _ => return Ok(access_token.to_string()),
    };

    if client_id.is_empty() || client_secret.is_empty() {
        return Ok(access_token.to_string());
    }

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", *refresh_token),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
    ];

    let res = client.post(token_url).form(&params).send().await;

    if let Ok(response) = res {
        if response.status().is_success() {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(new_access) = json.get("access_token").and_then(|t| t.as_str()) {
                    return Ok(new_access.to_string());
                }
            }
        }
    }

    Ok(access_token.to_string())
}

async fn get_or_create_google_drive_folder(
    client: &reqwest::Client,
    token_store: &str,
) -> Result<String, String> {
    let token = get_valid_access_token(client, token_store, Some("google-drive")).await?;

    let query = "name = 'hydra-save-companion' and mimeType = 'application/vnd.google-apps.folder' and trashed = false";
    let search_res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .query(&[("q", query)])
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau recherche dossier Google Drive : {}", e))?;

    if !search_res.status().is_success() {
        let err_body = search_res.text().await.unwrap_or_default();
        return Err(format!(
            "Erreur API Google (Recherche dossier) : {}",
            err_body
        ));
    }

    let search_json = search_res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Erreur parsing JSON recherche : {}", e))?;

    if let Some(files) = search_json.get("files").and_then(|f| f.as_array()) {
        if let Some(folder) = files.first() {
            return Ok(folder
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string());
        }
    }

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
        .map_err(|e| format!("Erreur réseau création dossier Google Drive : {}", e))?;

    if !create_res.status().is_success() {
        let err_body = create_res.text().await.unwrap_or_default();
        return Err(format!(
            "Erreur API Google (Création dossier) : {}",
            err_body
        ));
    }

    let create_json = create_res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Erreur parsing JSON création : {}", e))?;

    create_json
        .get("id")
        .and_then(|i| i.as_str())
        .ok_or("ID du dossier introuvable après création".to_string())
        .map(|s| s.to_string())
}

async fn get_or_create_dropbox_folder(
    client: &reqwest::Client,
    token_store: &str,
) -> Result<String, String> {
    let token = get_valid_access_token(client, token_store, Some("dropbox")).await?;
    let folder_path = "/hydra-save-companion";

    // 1. Vérification de l'existence du dossier via get_metadata
    let check_body = serde_json::json!({
        "path": folder_path
    });

    let check_res = client
        .post("https://api.dropboxapi.com/2/files/get_metadata")
        .bearer_auth(&token)
        .header("Content-Type", "application/json")
        .json(&check_body)
        .send()
        .await;

    if let Ok(res) = check_res {
        if res.status().is_success() {
            return Ok(folder_path.to_string());
        }
    }

    // 2. Création du dossier si inexistant
    let create_body = serde_json::json!({
        "path": folder_path,
        "autorename": false
    });

    let create_res = client
        .post("https://api.dropboxapi.com/2/files/create_folder_v2")
        .bearer_auth(&token)
        .header("Content-Type", "application/json")
        .json(&create_body)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau création dossier Dropbox : {}", e))?;

    if create_res.status().is_success() || create_res.status().as_u16() == 409 {
        Ok(folder_path.to_string())
    } else {
        let err_body = create_res.text().await.unwrap_or_default();
        Err(format!(
            "Erreur API Dropbox (Création dossier) : {}",
            err_body
        ))
    }
}

async fn get_or_create_proton_folder(
    client: &reqwest::Client,
    token_store: &str,
) -> Result<String, String> {
    // 1. Vérification / Création dans le dossier local Proton Drive
    let local_proton_base = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
    } else {
        std::env::var("HOME")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
    };

    if let Some(base) = local_proton_base {
        if base.exists() {
            let dest_dir = base.join("hydra-save-companion");
            let _ = std::fs::create_dir_all(&dest_dir);
            return Ok(dest_dir.to_string_lossy().into_owned());
        }
    }

    // 2. Si pas de dossier local, test de connexion sur l'API
    let token = get_valid_access_token(client, token_store, Some("proton-drive")).await?;
    if token.is_empty() {
        return Err(
            "Jeton d'accès Proton Drive manquant et aucun dossier local 'Proton Drive' trouvé."
                .to_string(),
        );
    }

    let api_res = client
        .get("https://mail-api.proton.me/drive/volumes")
        .bearer_auth(&token)
        .header("x-pm-appversion", "Other")
        .header("x-pm-locale", "fr_FR")
        .send()
        .await
        .map_err(|e| format!("Erreur réseau Proton Drive : {}", e))?;

    if api_res.status().is_success() {
        Ok("/hydra-save-companion".to_string())
    } else {
        let err_body = api_res.text().await.unwrap_or_default();
        Err(format!(
            "Erreur API Proton Drive (Vérification dossier) : {}",
            err_body
        ))
    }
}

pub async fn get_or_create_drive_folder(
    client: &reqwest::Client,
    token_store: &str,
    provider: Option<&str>,
) -> Result<String, String> {
    let p = provider.unwrap_or("google-drive").to_lowercase();
    match p.as_str() {
        "google-drive" | "gdrive" | "google" => {
            get_or_create_google_drive_folder(client, token_store).await
        }
        "dropbox" => get_or_create_dropbox_folder(client, token_store).await,
        "proton-drive" | "proton" => get_or_create_proton_folder(client, token_store).await,
        other => Err(format!(
            "Fournisseur cloud '{}' non supporté pour la création de dossier",
            other
        )),
    }
}

pub async fn check_sync_google_drive(
    token: String,
    file_name: String,
    local_modified_info: Option<(chrono::DateTime<chrono::Utc>, String)>,
) -> Result<SyncStatusResult, String> {
    use chrono::{DateTime, Local, Timelike, Utc};

    let client = reqwest::Client::new();
    let access_token = get_valid_access_token(&client, &token, Some("google-drive")).await?;
    let folder_id = get_or_create_drive_folder(&client, &token, Some("google-drive")).await?;

    let clean_title = file_name.trim_end_matches(".zip");
    let query = format!(
        "(name = '{}' or name contains '{}_') and mimeType = 'application/zip' and trashed = false and '{}' in parents",
        file_name.replace('\'', "\\'"),
        clean_title.replace('\'', "\\'"),
        folder_id
    );

    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,modifiedTime,name,appProperties)&orderBy=modifiedTime desc",
        urlencoding::encode(&query)
    );

    let res = client
        .get(&url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau vérification sync (Google Drive) : {}", e))?;

    let status = res.status();
    let body = res.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Erreur API Google Drive : {}", body));
    }

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Erreur parsing JSON : {}", e))?;

    let mut backups: Vec<RemoteBackupInfo> = Vec::new();

    if let Some(files_array) = json.get("files").and_then(|f| f.as_array()) {
        for file in files_array {
            let id = file
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let name = file
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let mod_time_str = file
                .get("modifiedTime")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            let formatted_time = DateTime::parse_from_rfc3339(mod_time_str)
                .map(|dt| {
                    dt.with_timezone(&Local)
                        .format("%d/%m/%Y %H:%M")
                        .to_string()
                })
                .unwrap_or_else(|_| mod_time_str.to_string());

            backups.push(RemoteBackupInfo {
                file_id: id,
                name,
                modified_time: formatted_time,
            });
        }

        if let Some(file) = files_array.first() {
            let cloud_modified_utc = file
                .get("modifiedTime")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| {
                    let utc = dt.with_timezone(&Utc);
                    utc.with_second(0).unwrap().with_nanosecond(0).unwrap()
                })
                .unwrap_or_else(Utc::now);

            let cloud_str = cloud_modified_utc
                .with_timezone(&Local)
                .format("%d/%m/%Y %H:%M")
                .to_string();

            return compare_and_build_result(
                local_modified_info,
                cloud_modified_utc,
                cloud_str,
                backups,
            );
        }
    }

    let local_str = local_modified_info
        .map(|(_, s)| s)
        .unwrap_or_else(|| "Jamais".to_string());
    Ok(SyncStatusResult {
        status: "NotFound".to_string(),
        localTime: local_str,
        cloudTime: "Jamais".to_string(),
        backups,
    })
}

pub async fn check_sync_dropbox(
    token: String,
    file_name: String,
    local_modified_info: Option<(chrono::DateTime<chrono::Utc>, String)>,
) -> Result<SyncStatusResult, String> {
    use chrono::{DateTime, Local, Timelike, Utc};

    let client = reqwest::Client::new();
    let access_token = get_valid_access_token(&client, &token, Some("dropbox")).await?;

    let clean_title = file_name.trim_end_matches(".zip");
    let mut backups: Vec<RemoteBackupInfo> = Vec::new();
    let mut latest_file: Option<(DateTime<Utc>, String)> = None;

    // Lister les fichiers du dossier pour trouver toutes les sauvegardes liées au jeu
    let list_body = serde_json::json!({
        "path": "/hydra-save-companion",
        "recursive": false
    });

    let res = client
        .post("https://api.dropboxapi.com/2/files/list_folder")
        .bearer_auth(&access_token)
        .json(&list_body)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau vérification sync (Dropbox) : {}", e))?;

    if res.status().is_success() {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(entries) = json.get("entries").and_then(|e| e.as_array()) {
                let mut matched_entries: Vec<(DateTime<Utc>, String, String, String)> = Vec::new();

                for entry in entries {
                    if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                        let matches_main = name.eq_ignore_ascii_case(&file_name);
                        let matches_prefix = name
                            .to_lowercase()
                            .starts_with(&format!("{}_", clean_title.to_lowercase()))
                            && name.ends_with(".zip");

                        if matches_main || matches_prefix {
                            let id = entry
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            let date_str = entry
                                .get("server_modified")
                                .or_else(|| entry.get("client_modified"))
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();

                            let dt_utc = DateTime::parse_from_rfc3339(date_str)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now());

                            let formatted_time = dt_utc
                                .with_timezone(&Local)
                                .format("%d/%m/%Y %H:%M")
                                .to_string();

                            matched_entries.push((dt_utc, id, name.to_string(), formatted_time));
                        }
                    }
                }

                // Trier par date décroissante
                matched_entries.sort_by(|a, b| b.0.cmp(&a.0));

                if let Some((first_utc, _, _, first_time_str)) = matched_entries.first() {
                    let utc_truncated = first_utc
                        .with_second(0)
                        .unwrap()
                        .with_nanosecond(0)
                        .unwrap();
                    latest_file = Some((utc_truncated, first_time_str.clone()));
                }

                for (_, id, name, formatted_time) in matched_entries {
                    backups.push(RemoteBackupInfo {
                        file_id: id,
                        name,
                        modified_time: formatted_time,
                    });
                }
            }
        }
    }

    if let Some((cloud_modified_utc, cloud_str)) = latest_file {
        return compare_and_build_result(
            local_modified_info,
            cloud_modified_utc,
            cloud_str,
            backups,
        );
    }

    let local_str = local_modified_info
        .map(|(_, s)| s)
        .unwrap_or_else(|| "Jamais".to_string());
    Ok(SyncStatusResult {
        status: "NotFound".to_string(),
        localTime: local_str,
        cloudTime: "Jamais".to_string(),
        backups,
    })
}

pub async fn check_sync_proton_drive(
    file_name: String,
    local_modified_info: Option<(chrono::DateTime<chrono::Utc>, String)>,
) -> Result<SyncStatusResult, String> {
    use chrono::{DateTime, Local, Timelike, Utc};
    use std::fs;

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let proton_dir = std::path::PathBuf::from(&home)
        .join("Proton Drive")
        .join("hydra-save-companion");

    let clean_title = file_name.trim_end_matches(".zip");
    let mut backups: Vec<RemoteBackupInfo> = Vec::new();
    let mut matched_files: Vec<(DateTime<Utc>, String, String, String)> = Vec::new();

    if proton_dir.exists() {
        if let Ok(entries) = fs::read_dir(&proton_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let matches_main = name.eq_ignore_ascii_case(&file_name);
                let matches_prefix = name
                    .to_lowercase()
                    .starts_with(&format!("{}_", clean_title.to_lowercase()))
                    && name.ends_with(".zip");

                if matches_main || matches_prefix {
                    let dt_utc: DateTime<Utc> = entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|st| st.into())
                        .unwrap_or_else(Utc::now);

                    let formatted_time = dt_utc
                        .with_timezone(&Local)
                        .format("%d/%m/%Y %H:%M")
                        .to_string();
                    matched_files.push((dt_utc, name.clone(), name, formatted_time));
                }
            }
        }
    }

    matched_files.sort_by(|a, b| b.0.cmp(&a.0));

    let latest_file = matched_files.first().map(|(utc, _, _, str_val)| {
        let utc_truncated = utc.with_second(0).unwrap().with_nanosecond(0).unwrap();
        (utc_truncated, str_val.clone())
    });

    for (_, id, name, formatted_time) in matched_files {
        backups.push(RemoteBackupInfo {
            file_id: id,
            name,
            modified_time: formatted_time,
        });
    }

    if let Some((cloud_modified_utc, cloud_str)) = latest_file {
        return compare_and_build_result(
            local_modified_info,
            cloud_modified_utc,
            cloud_str,
            backups,
        );
    }

    let local_str = local_modified_info
        .map(|(_, s)| s)
        .unwrap_or_else(|| "Jamais".to_string());
    Ok(SyncStatusResult {
        status: "NotFound".to_string(),
        localTime: local_str,
        cloudTime: "Jamais".to_string(),
        backups,
    })
}

pub async fn upload_to_google_drive(
    client: &reqwest::Client,
    token: &str,
    game_title: &str,
    zip_buffer: Vec<u8>,
    local_modified_str: &str,
    local_rfc3339: &str,
) -> Result<String, String> {
    let access_token = get_valid_access_token(client, token, Some("google-drive")).await?;
    let folder_id = get_or_create_drive_folder(client, token, Some("google-drive")).await?;

    let clean_title = game_title
        .replace(".zip", "")
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let file_name = format!("{}.zip", clean_title);

    let metadata = serde_json::json!({
        "name": file_name,
        "parents": [folder_id],
        "modifiedTime": local_rfc3339,
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

    let upload_url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&setModifiedDate=true";

    let upload_res = client
        .post(upload_url)
        .bearer_auth(&access_token)
        .multipart(multipart)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau upload Google Drive : {}", e))?;

    if !upload_res.status().is_success() {
        let err_text = upload_res.text().await.unwrap_or_default();
        return Err(format!(
            "Erreur lors de l'upload Google Drive : {}",
            err_text
        ));
    }

    let search_query = format!(
        "name = '{}' and '{}' in parents and trashed = false",
        file_name.replace('\'', "\\'"),
        folder_id
    );

    let search_res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(&access_token)
        .query(&[
            ("q", search_query.as_str()),
            ("fields", "files(id, name, createdTime)"),
            ("orderBy", "createdTime desc"),
        ])
        .send()
        .await;

    if let Ok(res) = search_res {
        if res.status().is_success() {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if let Some(files) = json["files"].as_array() {
                    if files.len() > 5 {
                        for old_file in &files[5..] {
                            if let Some(file_id) = old_file["id"].as_str() {
                                let delete_url = format!(
                                    "https://www.googleapis.com/drive/v3/files/{}",
                                    file_id
                                );
                                let _ = client
                                    .delete(&delete_url)
                                    .bearer_auth(&access_token)
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(format!(
        "Archive '{}' uploadée avec succès sur Google Drive !",
        file_name
    ))
}

pub async fn upload_to_dropbox(
    client: &reqwest::Client,
    token: &str,
    file_name: &str,
    zip_buffer: Vec<u8>,
    local_dt_utc: &chrono::DateTime<chrono::Utc>,
) -> Result<String, String> {
    let access_token = get_valid_access_token(client, token, Some("dropbox")).await?;
    let _ = get_or_create_drive_folder(client, token, Some("dropbox")).await;

    let dropbox_arg = serde_json::json!({
        "path": format!("/hydra-save-companion/{}", file_name),
        "mode": "overwrite",
        "autorename": false,
        "mute": false,
        "strict_conflict": false,
        "client_modified": local_dt_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    });

    let arg_str = serde_json::to_string(&dropbox_arg).unwrap_or_default();
    let ascii_arg: String = arg_str
        .chars()
        .map(|c| {
            if c.is_ascii() && c != '\r' && c != '\n' {
                c.to_string()
            } else {
                format!("\\u{:04x}", c as u32)
            }
        })
        .collect();

    let upload_res = client
        .post("https://content.dropboxapi.com/2/files/upload")
        .bearer_auth(&access_token)
        .header("Dropbox-API-Arg", &ascii_arg)
        .header("Content-Type", "application/octet-stream")
        .body(zip_buffer)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau Dropbox upload : {}", e))?;

    if upload_res.status().is_success() {
        Ok(format!(
            "Archive '{}' uploadée avec succès sur Dropbox !",
            file_name
        ))
    } else {
        let err_text = upload_res.text().await.unwrap_or_default();
        Err(format!("Erreur lors de l'upload Dropbox : {}", err_text))
    }
}

pub async fn upload_to_proton_drive(
    client: &reqwest::Client,
    token: &str,
    file_name: &str,
    zip_buffer: Vec<u8>,
) -> Result<String, String> {
    // 1. Détection prioritaire du dossier synchronisé Proton Drive local
    let local_proton_base = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
    } else {
        std::env::var("HOME")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
    };

    if let Some(base) = local_proton_base {
        if base.exists() {
            let dest_dir = base.join("hydra-save-companion");
            let _ = fs::create_dir_all(&dest_dir);
            let dest_file = dest_dir.join(file_name);
            fs::write(&dest_file, &zip_buffer)
                .map_err(|e| format!("Erreur écriture dans le dossier Proton Drive : {}", e))?;
            return Ok(format!(
                "Archive '{}' synchronisée avec succès dans le dossier local Proton Drive !",
                file_name
            ));
        }
    }

    // 2. Si aucun dossier local n'est détecté, vérification de l'accès API avec le jeton OAuth
    let access_token = get_valid_access_token(client, token, Some("proton-drive")).await?;
    if access_token.is_empty() {
        return Err("Aucun jeton d'accès Proton Drive valide et aucun dossier 'Proton Drive' local détecté.".to_string());
    }

    let api_res = client
        .get("https://mail-api.proton.me/drive/volumes")
        .bearer_auth(&access_token)
        .header("x-pm-appversion", "Other")
        .header("x-pm-locale", "fr_FR")
        .send()
        .await;

    match api_res {
        Ok(res) if res.status().is_success() => {
            Ok(format!(
                "Session Proton Drive active. (Pour le téléversement complet, le client Proton Drive de bureau est recommandé afin d'appliquer le chiffrement E2E)."
            ))
        }
        Ok(res) => {
            let err_text = res.text().await.unwrap_or_default();
            Err(format!(
                "Proton Drive nécessite un chiffrement de bout en bout (E2E). Veuillez lancer le client de bureau Proton Drive (dossier 'Proton Drive' introuvable). Détails : {}",
                err_text
            ))
        }
        Err(e) => Err(format!(
            "Erreur lors de la communication avec Proton Drive : {}",
            e
        )),
    }
}

pub async fn download_from_dropbox(
    client: &reqwest::Client,
    token: &str,
    file_name: &str,
    game_title: &str,
) -> Result<(Vec<u8>, std::time::SystemTime), String> {
    let access_token = get_valid_access_token(client, token, Some("dropbox")).await?;

    let dropbox_arg = serde_json::json!({
        "path": format!("/hydra-save-companion/{}", file_name)
    });

    let arg_str = serde_json::to_string(&dropbox_arg).unwrap_or_default();
    let ascii_arg: String = arg_str
        .chars()
        .map(|c| {
            if c.is_ascii() && c != '\r' && c != '\n' {
                c.to_string()
            } else {
                format!("\\u{:04x}", c as u32)
            }
        })
        .collect();

    let download_res = client
        .post("https://content.dropboxapi.com/2/files/download")
        .bearer_auth(&access_token)
        .header("Dropbox-API-Arg", &ascii_arg)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau téléchargement Dropbox : {}", e))?;

    if !download_res.status().is_success() {
        let err_text = download_res.text().await.unwrap_or_default();
        return Err(format!(
            "Aucune sauvegarde distante trouvée sur Dropbox pour '{}' : {}",
            game_title, err_text
        ));
    }

    let mut remote_system_time = std::time::SystemTime::now();
    if let Some(res_header) = download_res.headers().get("Dropbox-API-Result") {
        if let Ok(res_str) = res_header.to_str() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(res_str) {
                if let Some(client_modified) = json.get("client_modified").and_then(|v| v.as_str())
                {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(client_modified) {
                        remote_system_time = std::time::SystemTime::from(dt);
                    }
                } else if let Some(server_modified) =
                    json.get("server_modified").and_then(|v| v.as_str())
                {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(server_modified) {
                        remote_system_time = std::time::SystemTime::from(dt);
                    }
                }
            }
        }
    }

    let zip_bytes = download_res
        .bytes()
        .await
        .map_err(|e| format!("Erreur lecture octets ZIP Dropbox : {}", e))?
        .to_vec();

    Ok((zip_bytes, remote_system_time))
}

pub async fn download_from_proton_drive(
    client: &reqwest::Client,
    token: &str,
    file_name: &str,
    game_title: &str,
) -> Result<(Vec<u8>, std::time::SystemTime), String> {
    // 1. Détection prioritaire du dossier synchronisé Proton Drive local
    let local_proton_base = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
    } else {
        std::env::var("HOME")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
    };

    if let Some(base) = local_proton_base {
        let save_file = base.join("hydra-save-companion").join(file_name);
        if save_file.exists() {
            let metadata = fs::metadata(&save_file)
                .map_err(|e| format!("Erreur lecture métadonnées Proton Drive : {}", e))?;
            let modified = metadata
                .modified()
                .unwrap_or_else(|_| std::time::SystemTime::now());
            let bytes = fs::read(&save_file)
                .map_err(|e| format!("Erreur lecture fichier Proton Drive local : {}", e))?;
            return Ok((bytes, modified));
        }
    }

    // 2. Si non trouvé localement, vérification via l'API
    let access_token = get_valid_access_token(client, token, Some("proton-drive")).await?;
    if access_token.is_empty() {
        return Err(format!(
            "Aucune sauvegarde locale trouvée dans le dossier Proton Drive pour '{}' et jeton d'accès absent.",
            game_title
        ));
    }

    Err(format!(
        "Sauvegarde introuvable dans le dossier local Proton Drive pour '{}'. Veuillez vérifier que le fichier '{}.zip' est synchronisé sur votre machine.",
        game_title, file_name
    ))
}

pub async fn download_from_google_drive(
    client: &reqwest::Client,
    token: &str,
    file_name: &str,
    game_title: &str,
    target_file_id: Option<String>,
) -> Result<(Vec<u8>, std::time::SystemTime), String> {
    let access_token = get_valid_access_token(client, token, Some("google-drive")).await?;
    let folder_id = get_or_create_drive_folder(client, token, Some("google-drive")).await?;

    let file_id = match target_file_id {
        Some(id) => id,
        None => {
            let search_query = format!(
                "name = '{}' and '{}' in parents and trashed = false",
                file_name.replace('\'', "\\'"),
                folder_id
            );

            let search_res = client
                .get("https://www.googleapis.com/drive/v3/files")
                .bearer_auth(&access_token)
                .query(&[
                    ("q", search_query.as_str()),
                    ("fields", "files(id, modifiedTime)"),
                    ("orderBy", "createdTime desc"),
                ])
                .send()
                .await
                .map_err(|e| format!("Erreur recherche sauvegarde : {}", e))?;

            let search_json: serde_json::Value =
                search_res.json().await.map_err(|e| e.to_string())?;
            let files = search_json["files"].as_array().ok_or("Format invalide")?;
            let first = files
                .first()
                .ok_or_else(|| format!("Aucune sauvegarde trouvée pour '{}'", game_title))?;
            first["id"].as_str().ok_or("ID introuvable")?.to_string()
        }
    };

    let meta_res = client
        .get(&format!(
            "https://www.googleapis.com/drive/v3/files/{}?fields=modifiedTime",
            file_id
        ))
        .bearer_auth(&access_token)
        .send()
        .await;

    let drive_system_time = if let Ok(res) = meta_res {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(time_str) = json["modifiedTime"].as_str() {
                chrono::DateTime::parse_from_rfc3339(time_str)
                    .map(std::time::SystemTime::from)
                    .unwrap_or_else(|_| std::time::SystemTime::now())
            } else {
                std::time::SystemTime::now()
            }
        } else {
            std::time::SystemTime::now()
        }
    } else {
        std::time::SystemTime::now()
    };

    let download_url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media",
        file_id
    );
    let download_res = client
        .get(&download_url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau téléchargement : {}", e))?;

    if !download_res.status().is_success() {
        let err_text = download_res.text().await.unwrap_or_default();
        return Err(format!("Erreur téléchargement : {}", err_text));
    }

    let zip_bytes = download_res
        .bytes()
        .await
        .map_err(|e| e.to_string())?
        .to_vec();
    Ok((zip_bytes, drive_system_time))
}
