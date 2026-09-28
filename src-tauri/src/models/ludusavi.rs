use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct LudusaviManifest {
    pub games: HashMap<String, LudusaviGame>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviGame {
    pub files: Option<HashMap<String, LudusaviFileRule>>,
    pub steam: Option<LudusaviSteamInfo>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviFileRule {
    pub when: Option<Vec<LudusaviWhenCondition>>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviWhenCondition {
    pub os: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LudusaviSteamInfo {
    pub id: Option<u64>,
}