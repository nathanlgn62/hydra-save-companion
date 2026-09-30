use std::collections::HashMap;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct LudusaviManifest {
    #[serde(flatten)]
    pub games: HashMap<String, GameEntry>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GameEntry {
    pub files: Option<HashMap<String, FileRule>>,
    pub steam: Option<SteamInfo>,
    // Ajoute ici les autres champs si nécessaire selon ton code existant
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct FileRule {
    pub when: Option<Vec<Condition>>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Condition {
    pub os: Option<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SteamInfo {
    pub id: Option<u64>,
}
