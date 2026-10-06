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

#[tauri::command]
pub async fn login_google() -> Result<String, String> {
    let client_id = std::env::var("HSC_GC_ID")
        .map_err(|_| "La variable GOOGLE_CLIENT_ID est manquante".to_string())?;

    let client_secret = std::env::var("HSC_GS")
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
pub async fn upload_game_save_to_drive(
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

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

    // -------------------------------------------------------------
    // 1. RÉCUPÉRATION DE LA DATE LOCALE EXACTE (UTC & Format string)
    // -------------------------------------------------------------
    let local_modified_system_time =
        get_latest_modified_time(path).unwrap_or(std::time::SystemTime::now());

    let local_dt_utc: chrono::DateTime<chrono::Utc> = local_modified_system_time.into();
    let local_modified_str = local_dt_utc.format("%d/%m/%Y %H:%M").to_string();
    let local_rfc3339 = local_dt_utc.to_rfc3339();

    // -------------------------------------------------------------
    // 2. RECHERCHE D'UN FICHIER EXISTANT SUR LE CLOUD
    // -------------------------------------------------------------
    let search_query = format!(
        "name = '{}' and '{}' in parents and trashed = false",
        file_name.replace('\'', "\\'"),
        folder_id
    );

    let search_res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(&access_token)
        .query(&[("q", search_query.as_str()), ("fields", "files(id)")])
        .send()
        .await
        .map_err(|e| format!("Erreur recherche ancien fichier : {}", e))?;

    let existing_file_id = if search_res.status().is_success() {
        let json = search_res.json::<serde_json::Value>().await.ok();
        json.and_then(|j| {
            j["files"]
                .as_array()
                .and_then(|files| files.first())
                .and_then(|f| f["id"].as_str().map(|s| s.to_string()))
        })
    } else {
        None
    };

    // -------------------------------------------------------------
    // 3. CRÉATION DU ZIP
    // -------------------------------------------------------------
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
            let single_file_name = path
                .file_name()
                .ok_or("Nom de fichier invalide")?
                .to_string_lossy();
            zip.start_file(single_file_name, options)
                .map_err(|e| e.to_string())?;
            let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
    }

    // -------------------------------------------------------------
    // 4. PRÉPARATION DES MÉTADONNÉES ET DU MULTIPART
    // -------------------------------------------------------------
    let metadata = match existing_file_id {
        Some(_) => serde_json::json!({
            "name": file_name,
            "modifiedTime": local_rfc3339,
            "appProperties": {
                "localModified": local_modified_str
            }
        }),
        None => serde_json::json!({
            "name": file_name,
            "parents": [folder_id],
            "modifiedTime": local_rfc3339,
            "appProperties": {
                "localModified": local_modified_str
            }
        }),
    };

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

    // -------------------------------------------------------------
    // 5. UPLOAD (POST SI NOUVEAU, PATCH SI EXISTANT)
    // -------------------------------------------------------------
    let (upload_url, method) = match existing_file_id {
        Some(id) => (
            format!(
                "https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=multipart&setModifiedDate=true",
                id
            ),
            reqwest::Method::PATCH,
        ),
        None => (
            "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&setModifiedDate=true".to_string(),
            reqwest::Method::POST,
        ),
    };

    let upload_res = client
        .request(method, &upload_url)
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
pub async fn download_game_save_from_drive(
    token: String,
    game_title: String,
    save_path: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let access_token = get_valid_access_token(&client, &token).await?;
    let folder_id = get_or_create_drive_folder(&client, &token).await?;

    let file_name = format!(
        "{}.zip",
        game_title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    );

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
            ("fields", "files(id, name, modifiedTime)"),
        ])
        .send()
        .await
        .map_err(|e| format!("Erreur recherche sauvegarde distante : {}", e))?;

    if !search_res.status().is_success() {
        let err_text = search_res.text().await.unwrap_or_default();
        return Err(format!(
            "Erreur lors de la recherche du fichier distant : {}",
            err_text
        ));
    }

    let search_json: serde_json::Value = search_res
        .json()
        .await
        .map_err(|e| format!("Erreur parsage réponse Drive : {}", e))?;

    let file_obj = search_json["files"]
        .as_array()
        .and_then(|files| files.first())
        .ok_or_else(|| format!("Aucune sauvegarde distante trouvée pour '{}'", game_title))?;

    let file_id = file_obj["id"]
        .as_str()
        .ok_or_else(|| "ID du fichier distant introuvable".to_string())?;

    let remote_modified_time_str = file_obj["modifiedTime"].as_str().unwrap_or_default();

    let drive_system_time = chrono::DateTime::parse_from_rfc3339(remote_modified_time_str)
        .map(|dt| std::time::SystemTime::from(dt))
        .unwrap_or_else(|_| std::time::SystemTime::now());

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
        return Err(format!("Erreur lors du téléchargement : {}", err_text));
    }

    let zip_bytes = download_res
        .bytes()
        .await
        .map_err(|e| format!("Erreur lecture octets ZIP : {}", e))?;

    let target_path = Path::new(&save_path);

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

            // Écriture du fichier dans un bloc séparé pour qu'il soit fermé (drop) immédiatement après
            {
                let mut outfile = fs::File::create(&out_path)
                    .map_err(|e| format!("Erreur création fichier local : {}", e))?;

                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Erreur écriture fichier local : {}", e))?;
            }

            // Application de la date maintenant que le fichier est fermé sur le disque
            if let Ok(file_handle) = fs::File::options().write(true).open(&out_path) {
                let _ = file_handle.set_modified(drive_system_time);
            }
        }
    }

    // Appliquer également la date exacte de Google Drive au dossier cible (ou au parent si c'est un fichier unique)
    let folder_to_touch = if is_single_file_target {
        target_path.parent()
    } else {
        Some(target_path)
    };

    if let Some(p) = folder_to_touch {
        if p.exists() {
            if let Ok(folder_file) = fs::File::open(p) {
                let _ = folder_file.set_modified(drive_system_time);
            }
        }
    }

    Ok(format!(
        "Sauvegarde de '{}' restaurée avec succès !",
        game_title
    ))
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
    let mut params = vec![
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
