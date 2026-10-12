use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFileItem {
    pub id: String,
    pub modified_time: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFileList {
    pub files: Vec<DriveFileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudStorageInfo {
    pub provider: String,
    pub provider_name: String,
    pub used_bytes: u64,
    pub total_bytes: Option<u64>,
    pub used_formatted: String,
    pub total_formatted: Option<String>,
    pub free_bytes: Option<u64>,
    pub free_formatted: Option<String>,
}