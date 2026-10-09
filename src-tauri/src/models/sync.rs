use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct SyncStatusResult {
    pub status: String,
    #[serde(rename = "localTime")]
    pub local_time: String,
    #[serde(rename = "cloudTime")]
    pub cloud_time: String,
    #[serde(default)]
    pub backups: Vec<RemoteBackupInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RemoteBackupInfo {
    pub file_id: String,
    pub name: String,
    pub modified_time: String,
}
