use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveInfo {
    pub ludasavi_path_exists: bool, // Le chemin théorique trouvé via le manifeste existe sur le disque
    pub local_path_exists: bool,    // Le dossier contient de vraies données valides (non vide)
    pub resolved_path: Option<String>,
    pub last_modified: Option<String>,
}
