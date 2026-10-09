use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct LudasaviManifest {
    #[serde(flatten)]
    pub games: HashMap<String, GameEntry>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GameEntry {
    pub files: Option<HashMap<String, FileRule>>,
    pub steam: Option<SteamInfo>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct FileRule {
    pub when: Option<Vec<Condition>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Condition {
    pub os: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SteamInfo {
    pub id: Option<u64>,
}
