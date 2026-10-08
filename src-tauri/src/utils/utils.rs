use crate::models::sync::{RemoteBackupInfo, SyncStatusResult};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use zip::ZipArchive;

pub fn compare_and_build_result(
    local_modified_info: Option<(chrono::DateTime<chrono::Utc>, String)>,
    cloud_modified_utc: chrono::DateTime<chrono::Utc>,
    cloud_str: String,
    backups: Vec<RemoteBackupInfo>,
) -> Result<SyncStatusResult, String> {
    let (local_utc, local_str) = match local_modified_info {
        Some(info) => info,
        None => {
            return Ok(SyncStatusResult {
                status: "CloudNewer".to_string(),
                local_time: "Aucune".to_string(),
                cloud_time: cloud_str,
                backups,
            });
        }
    };

    if local_utc > cloud_modified_utc {
        Ok(SyncStatusResult {
            status: "LocalNewer".to_string(),
            local_time: local_str,
            cloud_time: cloud_str,
            backups,
        })
    } else if cloud_modified_utc > local_utc {
        Ok(SyncStatusResult {
            status: "CloudNewer".to_string(),
            local_time: local_str,
            cloud_time: cloud_str,
            backups,
        })
    } else {
        Ok(SyncStatusResult {
            status: "UpToDate".to_string(),
            local_time: local_str,
            cloud_time: cloud_str,
            backups,
        })
    }
}

pub fn create_save_zip(path: &Path) -> Result<Vec<u8>, String> {
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

pub fn restore_save_from_zip(
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
