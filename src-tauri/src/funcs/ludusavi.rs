use crate::models::ludusavi::LudusaviManifest;

pub fn get_or_fetch_manifest() -> Result<LudusaviManifest, String> {
    let cache_dir = env::temp_dir().join("hydra_companion");
    let manifest_path = cache_dir.join("ludusavi_manifest.json");

    let _ = fs::create_dir_all(&cache_dir);

    let should_download = if !manifest_path.exists() {
        true
    } else if let Ok(metadata) = fs::metadata(&manifest_path) {
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
                let _ = fs::write(&manifest_path, bytes);
            }
        }
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Impossible de lire le manifest : {}", e))?;

    serde_json::from_str::<LudusaviManifest>(&content)
        .map_err(|e| format!("Erreur de parsing du manifest Ludusavi : {}", e))
}
