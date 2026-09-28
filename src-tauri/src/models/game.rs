use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HydraGame {
    pub title: String,
    pub object_id: String,
    pub shop: Option<String>,
    pub executable_path: Option<String>,
    pub is_deleted: Option<bool>,
    pub icon_url: Option<String>,
    pub has_active_steam_import: Option<bool>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct GameProcessInfo {
    pub title: String,
    pub executable_name: String,
    pub save_path: Option<String>,
}