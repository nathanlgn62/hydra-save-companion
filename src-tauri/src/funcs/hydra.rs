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
