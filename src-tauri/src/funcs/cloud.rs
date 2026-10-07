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
                "742327849744-gsham3lda4i5pm37jmu2cj10c6mf215i.apps.googleusercontent.com".to_string()
            });
            let secret = std::env::var("HSC_GS").unwrap_or_else(|_| {
                "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".to_string()
            });
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

    let res = client
        .post(token_url)
        .form(&params)
        .send()
        .await;

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
        Err(format!("Erreur API Dropbox (Création dossier) : {}", err_body))
    }
}

async fn get_or_create_proton_folder(
    client: &reqwest::Client,
    token_store: &str,
) -> Result<String, String> {
    // 1. Vérification / Création dans le dossier local Proton Drive
    let local_proton_base = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").ok().map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
    } else {
        std::env::var("HOME").ok().map(|p| std::path::PathBuf::from(p).join("Proton Drive"))
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
        return Err("Jeton d'accès Proton Drive manquant et aucun dossier local 'Proton Drive' trouvé.".to_string());
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
        Err(format!("Erreur API Proton Drive (Vérification dossier) : {}", err_body))
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
        "dropbox" => {
            get_or_create_dropbox_folder(client, token_store).await
        }
        "proton-drive" | "proton" => {
            get_or_create_proton_folder(client, token_store).await
        }
        other => Err(format!(
            "Fournisseur cloud '{}' non supporté pour la création de dossier",
            other
        )),
    }
}
