use std::fs;
use std::io::Cursor;
use std::path::Path;
use tauri_plugin_oauth::start_with_config;
use tauri_plugin_oauth::OauthConfig;
use url::Url;
use zip::ZipArchive;

use crate::funcs::cloud::get_or_create_drive_folder;
use crate::funcs::cloud::get_valid_access_token;
use crate::funcs::hydra::get_latest_modified_time;

fn create_save_zip(path: &Path) -> Result<Vec<u8>, String> {
    let mut zip_buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut zip_buffer);
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
                    let mut f = fs::File::open(entry_path).map_err(|e| e.to_string())?;
                    std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                }
            }
        } else {
            let single_file_name = path
                .file_name()
                .ok_or("Nom de fichier invalide")?
                .to_string_lossy();
            zip.start_file(single_file_name, options)
                .map_err(|e| e.to_string())?;
            let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(zip_buffer)
}

async fn upload_to_google_drive(
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

async fn upload_to_dropbox(
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

async fn upload_to_proton_drive(
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

#[tauri::command]
pub async fn upload_game_save_to_drive(
    token: String,
    game_title: String,
    save_path: String,
    provider: Option<String>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let provider_name = provider.as_deref().unwrap_or("google-drive").to_lowercase();

    let path = Path::new(&save_path);
    if !path.exists() {
        return Err(format!(
            "Le chemin de sauvegarde local est introuvable : {}",
            save_path
        ));
    }

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    // 1. Récupération de la date locale exacte
    let local_modified_system_time =
        get_latest_modified_time(path).unwrap_or(std::time::SystemTime::now());
    let local_dt_utc: chrono::DateTime<chrono::Utc> = local_modified_system_time.into();
    let local_modified_str = local_dt_utc.format("%d/%m/%Y %H:%M").to_string();
    let local_rfc3339 = local_dt_utc.to_rfc3339();

    // 2. Création de l'archive ZIP
    let zip_buffer = create_save_zip(path)?;

    // 3. Routage vers le bon fournisseur Cloud
    match provider_name.as_str() {
        "google-drive" | "gdrive" | "google" => {
            upload_to_google_drive(
                &client,
                &token,
                &file_name,
                zip_buffer,
                &local_modified_str,
                &local_rfc3339,
            )
            .await
        }
        "dropbox" => {
            upload_to_dropbox(&client, &token, &file_name, zip_buffer, &local_dt_utc).await
        }
        "proton-drive" | "proton" => {
            upload_to_proton_drive(&client, &token, &file_name, zip_buffer).await
        }
        other => Err(format!(
            "Fournisseur cloud '{}' non supporté pour l'upload.",
            other
        )),
    }
}

async fn download_from_dropbox(
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

async fn download_from_proton_drive(
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

fn restore_save_from_zip(
    zip_bytes: Vec<u8>,
    save_path: &str,
    remote_time: std::time::SystemTime,
) -> Result<(), String> {
    let target_path = Path::new(save_path);

    if target_path.exists() {
        if target_path.is_file() {
            fs::remove_file(target_path)
                .map_err(|e| format!("Impossible de supprimer l'ancien fichier local : {}", e))?;
        } else if target_path.is_dir() {
            fs::remove_dir_all(target_path)
                .map_err(|e| format!("Impossible de nettoyer le dossier local : {}", e))?;
        }
    }

    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Archive ZIP invalide ou corrompue : {}", e))?;

    let is_single_file_target = target_path.extension().is_some();

    if is_single_file_target {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Erreur création dossier parent : {}", e))?;
        }
    } else {
        fs::create_dir_all(target_path)
            .map_err(|e| format!("Erreur création dossier cible : {}", e))?;
    }

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Erreur lecture entrée ZIP #{}: {}", i, e))?;

        let out_path = if is_single_file_target {
            target_path.to_path_buf()
        } else {
            match file.enclosed_name() {
                Some(path) => target_path.join(path),
                None => continue,
            }
        };

        if file.name().ends_with('/') {
            if !is_single_file_target {
                fs::create_dir_all(&out_path)
                    .map_err(|e| format!("Erreur création sous-dossier : {}", e))?;
            }
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Erreur création dossier parent : {}", e))?;
                }
            }

            {
                let mut outfile = fs::File::create(&out_path)
                    .map_err(|e| format!("Erreur création fichier local : {}", e))?;

                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Erreur écriture fichier local : {}", e))?;
            }

            if let Ok(file_handle) = fs::File::options().write(true).open(&out_path) {
                let _ = file_handle.set_modified(remote_time);
            }
        }
    }

    let folder_to_touch = if is_single_file_target {
        target_path.parent()
    } else {
        Some(target_path)
    };

    if let Some(p) = folder_to_touch {
        if p.exists() {
            if let Ok(folder_file) = fs::File::open(p) {
                let _ = folder_file.set_modified(remote_time);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn download_game_save_from_drive(
    token: String,
    game_title: String,
    save_path: String,
    provider: Option<String>,
    file_id: Option<String>, // Paramètre optionnel pour cibler un historique précis
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let provider_name = provider.as_deref().unwrap_or("google-drive").to_lowercase();

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    let (zip_bytes, remote_system_time) = match provider_name.as_str() {
        "google-drive" | "gdrive" | "google" => {
            download_from_google_drive(&client, &token, &file_name, &game_title, file_id).await?
        }
        "dropbox" => download_from_dropbox(&client, &token, &file_name, &game_title).await?,
        "proton-drive" | "proton" => {
            download_from_proton_drive(&client, &token, &file_name, &game_title).await?
        }
        other => return Err(format!("Fournisseur cloud '{}' non supporté.", other)),
    };

    restore_save_from_zip(zip_bytes, &save_path, remote_system_time)?;

    Ok(format!(
        "Sauvegarde de '{}' restaurée avec succès !",
        game_title
    ))
}

async fn download_from_google_drive(
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

#[tauri::command]
pub async fn login_cloud(provider: String) -> Result<String, String> {
    let (client_id_env, client_secret_env, auth_base_url, token_url, scope) = match provider
        .as_str()
    {
        "google-drive" => (
            "HSC_GC_ID",
            "HSC_GS",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            Some("https://www.googleapis.com/auth/drive.file"),
        ),
        "dropbox" => (
            "HSC_DBX_ID",
            "HSC_DBX_SECRET",
            "https://www.dropbox.com/oauth2/authorize",
            "https://api.dropbox.com/oauth2/token",
            None, // Dropbox gère ses scopes différemment ou dans l'App Console
        ),
        "proton-drive" => (
            "HSC_PROTON_ID",
            "HSC_PROTON_SECRET",
            "https://account.proton.me/oauth/authorize", // URL indicative selon l'API Proton
            "https://account.proton.me/oauth/token",
            Some("proton.drive"),
        ),
        "mega" => {
            return Err("Mega ne supporte pas l'OAuth standard de cette manière, gère-le via un identifiant/mot de passe ou l'API SDK".into());
        }
        _ => return Err("Fournisseur cloud inconnu".into()),
    };

    let client_id = std::env::var(client_id_env).map_err(|_| {
        format!(
            "La variable d'environnement pour l'ID de {} est manquante",
            provider
        )
    })?;
    let client_secret = std::env::var(client_secret_env).map_err(|_| {
        format!(
            "La variable d'environnement pour le Secret de {} est manquante",
            provider
        )
    })?;

    let redirect_uri = "http://localhost:8000";

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

    let mut auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&access_type=offline",
        auth_base_url, client_id, redirect_uri
    );

    if let Some(s) = scope {
        auth_url.push_str(&format!("&scope={}", s));
    }

    if provider == "dropbox" {
        auth_url.push_str("&token_access_type=offline");
    }

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
    let params = vec![
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri.to_string()),
        ("grant_type", "authorization_code".to_string()),
    ];

    let res = client
        .post(token_url)
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
