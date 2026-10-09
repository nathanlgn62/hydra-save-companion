use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::models::ludasavi::LudasaviManifest;



const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.yaml";

pub fn get_or_fetch_manifest() -> Result<LudasaviManifest, String> {
    let cache_dir = env::temp_dir().join("hydra_companion");
    let manifest_yaml_path = cache_dir.join("ludasavi_manifest.yaml");
    let manifest_json_path = cache_dir.join("ludasavi_manifest.json");

    let _ = fs::create_dir_all(&cache_dir);

    // 1. Si le cache JSON compilé existe déjà et est récent, on le lit directement en ~20-50ms (contre plusieurs secondes en YAML)
    if manifest_json_path.exists() {
        if let Ok(metadata) = fs::metadata(&manifest_json_path) {
            let is_recent = metadata
                .modified()
                .map(|m| m.elapsed().map(|d| d.as_secs() < 7 * 86400).unwrap_or(false))
                .unwrap_or(false);

            if is_recent {
                if let Ok(content) = fs::read_to_string(&manifest_json_path) {
                    if let Ok(manifest) = serde_json::from_str::<LudasaviManifest>(&content) {
                        return Ok(manifest);
                    }
                }
            }
        }
    }

    let should_download = if !manifest_yaml_path.exists() {
        true
    } else if let Ok(metadata) = fs::metadata(&manifest_yaml_path) {
        if let Ok(modified) = metadata.modified() {
            modified
                .elapsed()
                .map(|d| d.as_secs() > 7 * 86400)
                .unwrap_or(true)
        } else {
            true
        }
    } else {
        true
    };

    if should_download {
        if let Ok(response) = reqwest::blocking::get(MANIFEST_URL) {
            if let Ok(bytes) = response.bytes() {
                let _ = fs::write(&manifest_yaml_path, bytes);
            }
        }
    }

    let content = fs::read_to_string(&manifest_yaml_path)
        .map_err(|e| format!("Impossible de lire le manifest : {}", e))?;

    // Parsing YAML officiel de Ludasavi
    let manifest = serde_yaml::from_str::<LudasaviManifest>(&content)
        .map_err(|e| format!("Erreur de parsing du manifest Ludasavi YAML : {}", e))?;

    // Sauvegarde en cache JSON pour les lancements suivants ultra-rapides
    if let Ok(json_str) = serde_json::to_string(&manifest) {
        let _ = fs::write(&manifest_json_path, json_str);
    }

    Ok(manifest)
}

pub fn resolve_ludasavi_placeholders(path_str: &str, app_id: Option<&str>) -> String {
    let mut resolved = path_str.to_string();

    #[cfg(target_os = "windows")]
    {
        let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users".to_string());

        // Utilisation de PathBuf pour éviter les erreurs de slashs/antislashs
        let documents = PathBuf::from(&user_profile)
            .join("Documents")
            .to_string_lossy()
            .into_owned();
        let appdata = env::var("APPDATA").unwrap_or_default();
        let localappdata = env::var("LOCALAPPDATA").unwrap_or_default();
        let local_low = PathBuf::from(&user_profile)
            .join("AppData")
            .join("LocalLow")
            .to_string_lossy()
            .into_owned();
        let saved_games = PathBuf::from(&user_profile)
            .join("Saved Games")
            .to_string_lossy()
            .into_owned();
        let public = env::var("PUBLIC").unwrap_or_else(|_| r"C:\Users\Public".to_string());

        resolved = resolved.replace("<winDocuments>", &documents);
        resolved = resolved.replace("<winAppData>", &appdata);
        resolved = resolved.replace("<winLocalAppData>", &localappdata);
        resolved = resolved.replace("<winLocalAppDataLow>", &local_low);
        resolved = resolved.replace("<winSavedGames>", &saved_games);
        resolved = resolved.replace("<winPublic>", &public);
        resolved = resolved.replace("<winProgramData>", r"C:\ProgramData");

        if resolved.contains("<storeUserId>") || resolved.contains("<steamUser>") {
            resolved = resolved.replace("<storeUserId>", "*");
            resolved = resolved.replace("<steamUser>", "*");
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

pub fn resolve_path_pattern(pattern: &str) -> Option<PathBuf> {
    let path = PathBuf::from(pattern);

    if path.exists() {
        return Some(path);
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('\\').collect();
        let mut current_base = if !parts.is_empty() && parts[0].ends_with(':') {
            PathBuf::from(format!("{}\\", parts[0]))
        } else if !parts.is_empty() {
            PathBuf::from(parts[0])
        } else {
            PathBuf::new()
        };

        for part in &parts[1..] {
            if part.is_empty() {
                continue;
            }
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

// Cache global pour le manifeste Ludasavi
pub fn get_cached_manifest() -> Result<&'static LudasaviManifest, String> {
    static MANIFEST_CACHE_RES: OnceLock<Result<crate::models::ludasavi::LudasaviManifest, String>> =
        OnceLock::new();

    let res = MANIFEST_CACHE_RES.get_or_init(get_or_fetch_manifest);

    res.as_ref().map_err(|e| e.clone())
}
