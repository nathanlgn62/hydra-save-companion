use serde::{Deserialize, Serialize};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SyncStatusResult {
    pub status: String,
    pub localTime: String,
    pub cloudTime: String,
}