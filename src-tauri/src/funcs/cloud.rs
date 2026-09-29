pub async fn get_valid_access_token(
    client: &reqwest::Client,
    token_store: &str,
) -> Result<String, String> {
    let parts: Vec<&str> = token_store.split('|').collect();
    let access_token = parts.first().unwrap_or(&"");
    let refresh_token = parts.get(1).unwrap_or(&"");

    if refresh_token.is_empty() {
        return Ok(access_token.to_string());
    }

    let client_id = "742327849744-gsham3lda4i5pm37jmu2cj10c6mf215i.apps.googleusercontent.com";
    let client_secret = "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";

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", *refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
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

pub async fn get_or_create_drive_folder(
    client: &reqwest::Client,
    token_store: &str,
) -> Result<String, String> {
    let token = get_valid_access_token(client, token_store).await?;

    let query = "name = 'hydra-save-companion' and mimeType = 'application/vnd.google-apps.folder' and trashed = false";
    let search_res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .query(&[("q", query)])
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau recherche dossier : {}", e))?;

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
        .map_err(|e| format!("Erreur réseau création dossier : {}", e))?;

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
