use std::path::Path;
use tauri::Manager;
use tauri_plugin_oauth::start_with_config;
use tauri_plugin_oauth::OauthConfig;
use url::Url;

use crate::funcs::cloud::download_from_dropbox;
use crate::funcs::cloud::download_from_google_drive;
use crate::funcs::cloud::download_from_proton_drive;
use crate::funcs::cloud::upload_to_dropbox;
use crate::funcs::cloud::upload_to_google_drive;
use crate::funcs::cloud::upload_to_proton_drive;
use crate::funcs::hydra::get_latest_modified_time;
use crate::utils::utils::create_save_zip;
use crate::utils::utils::restore_save_from_zip;

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

#[tauri::command]
pub async fn login_cloud(app: tauri::AppHandle, provider: String) -> Result<String, String> {
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

    let html_response = r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="utf-8">
    <title>Connexion réussie</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            background-color: #020617;
            color: #f8fafc;
            font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            text-align: center;
        }
        .card {
            background-color: #0f172a;
            border: 1px solid #1e293b;
            border-radius: 16px;
            padding: 36px 44px;
            box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
            max-width: 440px;
        }
        .icon {
            width: 52px;
            height: 52px;
            border-radius: 50%;
            background-color: rgba(16, 185, 129, 0.15);
            color: #10b981;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 18px auto;
            font-size: 26px;
            font-weight: bold;
        }
        h1 { font-size: 20px; margin: 0 0 10px 0; color: #f1f5f9; }
        p { font-size: 14px; margin: 0; color: #94a3b8; line-height: 1.5; }
        .badge { margin-top: 18px; display: inline-block; padding: 6px 14px; background: #1e293b; border-radius: 8px; font-size: 12px; color: #64748b; }
    </style>
</head>
<body>
    <div class="card">
        <div class="icon">✓</div>
        <h1>Connexion réussie !</h1>
        <p>Votre compte a été connecté avec succès à <strong>Hydra Save Companion</strong>.</p>
        <div class="badge">Retour automatique sur l'application...</div>
    </div>
    <script>
        window.onload = function() {
            setTimeout(function() {
                window.open('', '_self', '');
                window.close();
            }, 300);
        };
    </script>
</body>
</html>"#;

    let config = OauthConfig {
        ports: Some(vec![8000, 8001, 8002]),
        response: Some(html_response.into()),
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

    // Refocus immédiat de la fenêtre de l'application dès réception du code de redirection
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

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

    // Assure que l'application reste bien au premier plan une fois les jetons obtenus
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    Ok(format!("{}|{}", access_token, refresh_token))
}

#[tauri::command]
pub async fn get_cloud_storage_info(
    token: String,
    provider: Option<String>,
) -> Result<crate::models::cloud::CloudStorageInfo, String> {
    let client = reqwest::Client::new();
    let provider_name = provider.as_deref().unwrap_or("google-drive");
    crate::funcs::cloud::get_cloud_storage_info(&client, &token, Some(provider_name)).await
}
