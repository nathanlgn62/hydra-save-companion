use serde::{Deserialize, Serialize};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SyncStatusResult {
    pub status: String,
    pub localTime: String,
    pub cloudTime: String,
    #[serde(default)]
    pub backups: Vec<RemoteBackupInfo>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RemoteBackupInfo {
    pub file_id: String,
    pub name: String,
    pub modified_time: String,
}
