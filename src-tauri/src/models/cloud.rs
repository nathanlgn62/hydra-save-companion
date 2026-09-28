use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFileItem {
    id: String,
    modified_time: String,
}

#[derive(Deserialize)]
pub struct DriveFileList {
    files: Vec<DriveFileItem>,
}