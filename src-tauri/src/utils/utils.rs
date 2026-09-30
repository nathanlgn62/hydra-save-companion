use chrono::DateTime;

pub fn get_fallback_cloud_time(file: &serde_json::Value) -> String {
    use chrono::Local;
    if let Some(modified_time_str) = file.get("modifiedTime").and_then(|t| t.as_str()) {
        if let Ok(cloud_modified) = DateTime::parse_from_rfc3339(modified_time_str) {
            return cloud_modified
                .with_timezone(&Local)
                .format("%d/%m/%Y %H:%M")
                .to_string();
        }
    }
    "Jamais".to_string()
}
