use rusty_leveldb::{LdbIterator, Options, DB};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::models::ludasavi::{GameEntry, LudasaviManifest};

const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.yaml";

static RESOLVED_GAMES_CACHE: OnceLock<Mutex<HashMap<String, Option<GameEntry>>>> = OnceLock::new();

fn get_resolved_cache() -> &'static Mutex<HashMap<String, Option<GameEntry>>> {
    RESOLVED_GAMES_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// S'assure que la base LevelDB locale des 53 000+ jeux est indexée sur le disque.
/// Les 53 000 entrées restent stockées sur disque et ne sont JAMAIS gardées en RAM.
pub fn preload_manifest_file() -> Result<(), String> {
    let cache_dir = env::temp_dir().join("hydra_companion");
    let db_path = cache_dir.join("ludasavi_ldb");
    let manifest_yaml_path = cache_dir.join("ludasavi_manifest.yaml");
    let manifest_json_path = cache_dir.join("ludasavi_manifest.json");

    let _ = fs::create_dir_all(&cache_dir);

    // Si la base LevelDB existe déjà et est récente (< 7 jours), elle est prête
    if db_path.exists() {
        if let Ok(metadata) = fs::metadata(&db_path) {
            let is_recent = metadata
                .modified()
                .map(|m| m.elapsed().map(|d| d.as_secs() < 7 * 86400).unwrap_or(false))
                .unwrap_or(false);

            if is_recent {
                return Ok(());
            }
        }
    }

    // Téléchargement du YAML officiel si nécessaire
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

    // Si on a déjà un JSON sur le disque, on l'utilise pour construire la LevelDB
    let games_map: Option<HashMap<String, GameEntry>> = if manifest_json_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_json_path) {
            serde_json::from_str::<LudasaviManifest>(&content)
                .map(|m| m.games)
                .ok()
        } else {
            None
        }
    } else if manifest_yaml_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_yaml_path) {
            serde_yaml::from_str::<LudasaviManifest>(&content)
                .map(|m| m.games)
                .ok()
        } else {
            None
        }
    } else {
        None
    };

    if let Some(games) = games_map {
        let _ = fs::remove_dir_all(&db_path);

        let mut opt = Options::default();
        opt.create_if_missing = true;

        if let Ok(mut db) = DB::open(&db_path, opt) {
            for (title, entry) in games {
                let title_clean = title.trim().to_lowercase();
                if let Ok(val_bytes) = serde_json::to_vec(&entry) {
                    let key = format!("t:{}", title_clean);
                    let _ = db.put(key.as_bytes(), &val_bytes);

                    if let Some(ref steam) = entry.steam {
                        if let Some(steam_id) = steam.id {
                            let s_key = format!("s:{}", steam_id);
                            let _ = db.put(s_key.as_bytes(), &val_bytes);
                        }
                    }
                }
            }
            let _ = db.flush();
        }
    }

    Ok(())
}

/// Recherche un jeu dans la base LevelDB locale sur disque.
/// ZÉRO jeu n'est conservé en RAM, seul le jeu demandé est lu et désérialisé.
pub fn find_game_in_manifest(title: &str, app_id: Option<&str>) -> Option<GameEntry> {
    let cache_key = format!("{}:{}", title.to_lowercase().trim(), app_id.unwrap_or(""));

    // 1. Vérification dans le micro-cache permanent des jeux résolus de l'utilisateur
    if let Ok(cache) = get_resolved_cache().lock() {
        if let Some(entry) = cache.get(&cache_key) {
            return entry.clone();
        }
    }

    let cache_dir = env::temp_dir().join("hydra_companion");
    let db_path = cache_dir.join("ludasavi_ldb");

    if !db_path.exists() {
        let _ = preload_manifest_file();
    }

    let mut opt = Options::default();
    opt.create_if_missing = false;

    let mut db = DB::open(&db_path, opt).ok()?;

    let title_clean = title.trim().to_lowercase();

    // 2. Recherche par clé de titre exacte dans LevelDB : O(1) sur disque
    let mut found_bytes = db.get(format!("t:{}", title_clean).as_bytes());

    // 3. Recherche par Steam AppID si présent
    if found_bytes.is_none() {
        if let Some(id) = app_id {
            found_bytes = db.get(format!("s:{}", id).as_bytes());
        }
    }

    // 4. Recherche par correspondance partielle si introuvable en direct
    if found_bytes.is_none() {
        if let Ok(mut iter) = db.new_iter() {
            while let Some((k, v)) = iter.next() {
                if k.starts_with(b"t:") {
                    if let Ok(k_str) = std::str::from_utf8(&k[2..]) {
                        if k_str.contains(&title_clean) || title_clean.contains(k_str) {
                            found_bytes = Some(v.into());
                            break;
                        }
                    }
                }
            }
        }
    }

    let entry: Option<GameEntry> = found_bytes.and_then(|bytes| {
        serde_json::from_slice::<GameEntry>(&bytes).ok()
    });

    // Mémorisation dans le micro-cache
    if let Ok(mut cache) = get_resolved_cache().lock() {
        cache.insert(cache_key, entry.clone());
    }

    entry
}

pub fn resolve_ludasavi_placeholders(path_str: &str, app_id: Option<&str>) -> String {
    let mut resolved = path_str.to_string();

    #[cfg(target_os = "windows")]
    {
        let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users".to_string());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_ludasavi_placeholders() {
        let path = "<home>/saves/<game>";
        let resolved = resolve_ludasavi_placeholders(path, Some("12345"));
        assert!(resolved.contains("12345"));
    }

    #[test]
    fn test_preload_and_find_game() {
        let cache_dir = env::temp_dir().join("hydra_companion");
        let db_path = cache_dir.join("ludasavi_ldb");

        if !db_path.exists() {
            let res = preload_manifest_file();
            if res.is_err() || !db_path.exists() {
                // Pas de connexion réseau ou cache non initialisé en environnement de test
                return;
            }
        }

        let _ = find_game_in_manifest("Cyberpunk 2077", Some("1091500"));
    }
}
