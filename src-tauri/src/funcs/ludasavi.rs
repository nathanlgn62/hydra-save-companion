use rusty_leveldb::{LdbIterator, Options, DB};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::models::ludasavi::{GameEntry, LudasaviManifest};

const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.yaml";

static MANIFEST_DB: OnceLock<Mutex<Option<DB>>> = OnceLock::new();
static RESOLVED_GAMES_CACHE: OnceLock<Mutex<HashMap<String, Option<GameEntry>>>> = OnceLock::new();

fn get_manifest_db() -> &'static Mutex<Option<DB>> {
    MANIFEST_DB.get_or_init(|| Mutex::new(None))
}

fn get_resolved_cache() -> &'static Mutex<HashMap<String, Option<GameEntry>>> {
    RESOLVED_GAMES_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clean_title_for_lookup(title: &str) -> String {
    title
        .replace(['™', '®', '©'], "")
        .trim()
        .to_lowercase()
}

/// S'assure que la base LevelDB locale des 53 000+ jeux est indexée sur le disque
/// et ouverte de manière thread-safe dans un singleton partagé.
pub fn preload_manifest_file() -> Result<(), String> {
    let cache_dir = env::temp_dir().join("hydra_companion");
    let db_path = cache_dir.join("ludasavi_ldb");
    let manifest_yaml_path = cache_dir.join("ludasavi_manifest.yaml");
    let manifest_json_path = cache_dir.join("ludasavi_manifest.json");

    let _ = fs::create_dir_all(&cache_dir);

    let db_mutex = get_manifest_db();
    let mut db_guard = db_mutex.lock().map_err(|e| e.to_string())?;

    if db_guard.is_some() {
        return Ok(());
    }

    // Si la base LevelDB existe déjà et est récente (< 7 jours), on l'ouvre une seule fois
    if db_path.exists() {
        if let Ok(metadata) = fs::metadata(&db_path) {
            let is_recent = metadata
                .modified()
                .map(|m| {
                    m.elapsed()
                        .map(|d| d.as_secs() < 7 * 86400)
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if is_recent {
                let mut opt = Options::default();
                opt.create_if_missing = false;
                if let Ok(db) = DB::open(&db_path, opt) {
                    *db_guard = Some(db);
                    return Ok(());
                }
            }
        }
    }

    // Libérer l'instance existante avant écriture/suppression
    *db_guard = None;

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
                let title_norm = clean_title_for_lookup(&title);
                if let Ok(val_bytes) = serde_json::to_vec(&entry) {
                    let key = format!("t:{}", title_clean);
                    let _ = db.put(key.as_bytes(), &val_bytes);

                    if title_norm != title_clean {
                        let norm_key = format!("t:{}", title_norm);
                        let _ = db.put(norm_key.as_bytes(), &val_bytes);
                    }

                    if let Some(ref steam) = entry.steam {
                        if let Some(steam_id) = steam.id {
                            let s_key = format!("s:{}", steam_id);
                            let _ = db.put(s_key.as_bytes(), &val_bytes);
                        }
                    }
                }
            }
            let _ = db.flush();
            *db_guard = Some(db);
        }
    }

    Ok(())
}

/// Recherche un jeu dans la base LevelDB locale sur disque.
/// ZÉRO jeu n'est conservé en RAM, seul le jeu demandé est lu et désérialisé.
pub fn find_game_in_manifest(title: &str, app_id: Option<&str>) -> Option<GameEntry> {
    let clean_title = clean_title_for_lookup(title);
    let cache_key = format!("{}:{}", clean_title, app_id.unwrap_or(""));

    // 1. Vérification dans le micro-cache permanent des jeux résolus de l'utilisateur
    if let Ok(cache) = get_resolved_cache().lock() {
        if let Some(entry) = cache.get(&cache_key) {
            return entry.clone();
        }
    }

    let cache_dir = env::temp_dir().join("hydra_companion");
    let db_path = cache_dir.join("ludasavi_ldb");

    let db_mutex = get_manifest_db();
    let mut db_guard = db_mutex.lock().ok()?;

    if db_guard.is_none() {
        if !db_path.exists() {
            drop(db_guard);
            let _ = preload_manifest_file();
            db_guard = db_mutex.lock().ok()?;
        } else {
            let mut opt = Options::default();
            opt.create_if_missing = false;
            match DB::open(&db_path, opt) {
                Ok(db) => *db_guard = Some(db),
                Err(e) => {
                    println!("[DEBUG] DB::open failed on existing db: {:?}. Retrying via preload_manifest_file...", e);
                    drop(db_guard);
                    let _ = preload_manifest_file();
                    db_guard = db_mutex.lock().ok()?;
                }
            }
        }
    }

    let db = db_guard.as_mut()?;

    // 2. Recherche prioritaire par Steam AppID si présent (identifiant unique et canonique)
    let mut found_bytes = None;
    if let Some(id) = app_id {
        let clean_id = id.trim();
        if !clean_id.is_empty() {
            found_bytes = db.get(format!("s:{}", clean_id).as_bytes());
        }
    }

    // 3. Recherche par clé de titre brute minuscule
    let raw_lower = title.trim().to_lowercase();
    if found_bytes.is_none() {
        found_bytes = db.get(format!("t:{}", raw_lower).as_bytes());
    }

    // 4. Recherche par titre normalisé (sans symboles ™, ®, ©, etc.)
    if found_bytes.is_none() && clean_title != raw_lower {
        found_bytes = db.get(format!("t:{}", clean_title).as_bytes());
    }

    // 5. Recherche par titre principal (avant sous-titre ':' ou ' - ')
    if found_bytes.is_none() {
        if let Some((main_title, _)) = clean_title.split_once(':').or_else(|| clean_title.split_once(" - ")) {
            let main_clean = main_title.trim();
            if main_clean.len() >= 3 {
                found_bytes = db.get(format!("t:{}", main_clean).as_bytes());
            }
        }
    }

    // 6. Recherche par itération / correspondance partielle si introuvable en direct
    if found_bytes.is_none() {
        if let Ok(mut iter) = db.new_iter() {
            let mut best_match: Option<(usize, Vec<u8>)> = None;
            while let Some((k, v)) = iter.next() {
                if k.starts_with(b"t:") {
                    if let Ok(k_str) = std::str::from_utf8(&k[2..]) {
                        let matches = if k_str == clean_title {
                            true
                        } else if k_str.contains(&clean_title) {
                            true
                        } else if clean_title.contains(k_str) && k_str.len() >= 5 && k_str.len() >= clean_title.len() / 2 {
                            true
                        } else {
                            false
                        };

                        if matches {
                            let diff = (k_str.len() as isize - clean_title.len() as isize).unsigned_abs();
                            if best_match.as_ref().map_or(true, |(best_diff, _)| diff < *best_diff) {
                                best_match = Some((diff, v));
                                if diff == 0 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if let Some((_, v)) = best_match {
                found_bytes = Some(v.into());
            }
        }
    }

    let entry: Option<GameEntry> =
        found_bytes.and_then(|bytes| serde_json::from_slice::<GameEntry>(&bytes).ok());

    // Mémorisation dans le micro-cache uniquement si le jeu est trouvé
    if let Some(ref e) = entry {
        if let Ok(mut cache) = get_resolved_cache().lock() {
            cache.insert(cache_key, Some(e.clone()));
        }
    }

    entry
}

pub fn resolve_ludasavi_placeholders(
    path_str: &str,
    app_id: Option<&str>,
    executable_path: Option<&str>,
) -> String {
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

    if resolved.contains("<base>") {
        if let Some(exec) = executable_path {
            let exec_p = Path::new(exec);
            let mut candidates = Vec::new();
            let mut curr = exec_p.parent();
            while let Some(parent) = curr {
                candidates.push(parent.to_path_buf());
                curr = parent.parent();
                if candidates.len() >= 4 {
                    break;
                }
            }

            let rel_suffix = resolved
                .replace("<base>", "")
                .trim_start_matches(['/', '\\'])
                .to_string();
            let prefix_part = rel_suffix.split('*').next().unwrap_or(&rel_suffix);
            let prefix_part = prefix_part.trim_end_matches(['/', '\\']);

            let mut matched_base = None;
            for cand in &candidates {
                let test_path = cand.join(prefix_part);
                if test_path.exists() {
                    matched_base = Some(cand.clone());
                    break;
                }
            }

            let base_dir = matched_base.unwrap_or_else(|| {
                if candidates.len() >= 2 {
                    let p1 = candidates[0].to_string_lossy().to_lowercase();
                    if p1.ends_with("launcher")
                        || p1.ends_with("bin")
                        || p1.ends_with("win64")
                        || p1.ends_with("binaries")
                    {
                        candidates[1].clone()
                    } else if candidates.len() >= 3
                        && (p1.ends_with("win64") || p1.ends_with("shipping"))
                    {
                        candidates[2].clone()
                    } else {
                        candidates[0].clone()
                    }
                } else if !candidates.is_empty() {
                    candidates[0].clone()
                } else {
                    PathBuf::new()
                }
            });

            resolved = resolved.replace("<base>", &base_dir.to_string_lossy());
        }
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

        let total_parts = parts.len();
        for (i, part) in parts[1..].iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            let is_last = (i + 2) == total_parts;

            if part.contains('*') {
                if let Ok(entries) = fs::read_dir(&current_base) {
                    let mut found = false;
                    for entry in entries.flatten() {
                        let is_dir = entry.path().is_dir();
                        if is_dir {
                            current_base = entry.path();
                            found = true;
                            break;
                        } else if is_last {
                            // Joker de fichier (ex: *.sav) : on retient le dossier parent
                            return Some(current_base);
                        }
                    }
                    if !found {
                        if is_last && current_base.exists() {
                            return Some(current_base);
                        }
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
        let resolved = resolve_ludasavi_placeholders(path, Some("12345"), None);
        assert!(resolved.contains("12345"));

        // Test <base> placeholder with MGS 4
        let mgs_path = "<base>/mgs4_savedata_win/<storeUserId>";
        let mgs_exec = "D:\\Games\\METAL GEAR SOLID 4 Guns of the Patriots Master Collection Version\\Launcher\\launcher.exe";
        let resolved_mgs = resolve_ludasavi_placeholders(mgs_path, Some("2492670"), Some(mgs_exec));
        println!("Resolved MGS path: {}", resolved_mgs);
        assert!(!resolved_mgs.contains("<base>"));
        assert!(resolved_mgs.contains("METAL GEAR SOLID 4"));
    }

    #[test]
    fn test_preload_and_find_game() {
        let _ = preload_manifest_file();
        let games = vec![
            ("STAR WARS Zero Company™", Some("2075800")),
            ("The Witcher 3: Wild Hunt", Some("292030")),
            (
                "METAL GEAR SOLID 4: Guns of the Patriots - Master Collection Version",
                Some("2492670"),
            ),
            ("1666: Amsterdam", Some("3949550")),
        ];

        // Test concurrent access from multiple threads
        println!("\nTesting concurrent access from 4 threads...");
        let handles: Vec<_> = games
            .into_iter()
            .map(|(t, id)| {
                std::thread::spawn(move || {
                    let res = find_game_in_manifest(t, id);
                    println!("Thread for '{}' => found: {}", t, res.is_some());
                    assert!(res.is_some(), "Game '{}' should be found!", t);
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
    }
}
