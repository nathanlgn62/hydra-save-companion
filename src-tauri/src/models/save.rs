use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveInfo {
    pub path_exists: bool,
    pub resolved_path: Option<String>,
    pub last_modified: Option<String>,
}