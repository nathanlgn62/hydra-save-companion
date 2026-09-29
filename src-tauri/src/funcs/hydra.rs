use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn get_hydra_db_dir() -> Result<PathBuf, String> {
    let base_dir = if cfg!(target_os = "windows") {
        let appdata = env::var("APPDATA").map_err(|_| "Variable APPDATA introuvable")?;
        PathBuf::from(appdata).join("hydralauncher")
    } else {
        let home = env::var("HOME").map_err(|_| "Variable HOME introuvable")?;
        let config_path = PathBuf::from(&home).join(".config/hydralauncher");
        if config_path.exists() {
            config_path
        } else {
            PathBuf::from(home).join(".local/share/hydralauncher")
        }
    };

    Ok(base_dir.join("hydra-db"))
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name == "LOCK" {
            continue;
        }
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(file_name))?;
        } else {
            let _ = fs::copy(entry.path(), dst.join(file_name));
        }
    }
    Ok(())
}

pub fn expand_path(path_str: &str) -> PathBuf {
    let mut expanded = path_str.to_string();

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            expanded = expanded.replace("%APPDATA%", &appdata);
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            expanded = expanded.replace("%LOCALAPPDATA%", &localappdata);
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            expanded = expanded.replace("%USERPROFILE%", &userprofile);
        }
        if let Ok(public) = std::env::var("PUBLIC") {
            expanded = expanded.replace("%PUBLIC%", &public);
        } else {
            expanded = expanded.replace("%PUBLIC%", r"C:\Users\Public");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            if expanded.starts_with('~') {
                expanded = expanded.replacen('~', &home, 1);
            }
            expanded = expanded.replace("$HOME", &home);
        }
    }

    PathBuf::from(expanded)
}

pub fn get_latest_modified_time(dir: &Path) -> Option<SystemTime> {
    if !dir.exists() {
        return None;
    }

    if dir.is_file() {
        return dir.metadata().ok()?.modified().ok();
    }

    let mut latest: Option<SystemTime> = None;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let time = if path.is_dir() {
                get_latest_modified_time(&path)
            } else {
                path.metadata().ok()?.modified().ok()
            };

            if let Some(t) = time {
                if latest.is_none() || t > latest.unwrap() {
                    latest = Some(t);
                }
            }
        }
    }

    latest
}

pub fn get_candidate_paths(app_id: Option<&str>, title: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let clean_title = title.replace(':', "").replace('/', "").replace('\\', "");

    if let Some(id) = app_id {
        if !id.trim().is_empty() {
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\RUNE\{}",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\RUNE\{}\remote",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\CODEX\{}",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\CODEX\{}\remote",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\TENOKE\{}",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\EMPRESS\{}\remote",
                id
            )));

            candidates.push(expand_path(&format!(
                r"%APPDATA%\Goldberg SteamEmu Saves\{}",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%APPDATA%\Goldberg SteamEmu Saves\{}\remote",
                id
            )));
            candidates.push(expand_path(&format!(r"%APPDATA%\FLT\{}", id)));
            candidates.push(expand_path(&format!(r"%APPDATA%\Razor1911Appdata\{}", id)));

            candidates.push(expand_path(&format!(
                r"C:\Program Files (x86)\Steam\userdata\*\*\remote"
            )));
        }
    }

    if title.to_lowercase().contains("witcher") {
        let tw3_ids = ["292030", "378120", "499450"];
        for id in tw3_ids {
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\RUNE\{}\remote",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\RUNE\{}",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\CODEX\{}\remote",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%PUBLIC%\Documents\Steam\CODEX\{}",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%APPDATA%\Goldberg SteamEmu Saves\{}\remote",
                id
            )));
            candidates.push(expand_path(&format!(
                r"%APPDATA%\Goldberg SteamEmu Saves\{}",
                id
            )));
        }

        candidates.push(expand_path(
            r"%USERPROFILE%\Documents\The Witcher 3\gamesaves",
        ));
        candidates.push(expand_path(
            r"%USERPROFILE%\Documents\The Witcher 3 Wild Hunt\gamesaves",
        ));
    }

    candidates.push(expand_path(&format!(
        r"%USERPROFILE%\Saved Games\{}",
        clean_title
    )));
    candidates.push(expand_path(&format!(
        r"%USERPROFILE%\Documents\{}\gamesaves",
        clean_title
    )));
    candidates.push(expand_path(&format!(
        r"%USERPROFILE%\Documents\{}",
        clean_title
    )));
    candidates.push(expand_path(&format!(
        r"%LOCALAPPDATA%\{}\Saved\SaveGames",
        clean_title
    )));
    candidates.push(expand_path(&format!(r"%LOCALAPPDATA%\{}", clean_title)));
    candidates.push(expand_path(&format!(r"%APPDATA%\{}", clean_title)));

    candidates
}
