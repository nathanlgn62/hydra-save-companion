import { invoke } from "@tauri-apps/api/core";

export async function openFolder(savePath?: string | null) {
  if (!savePath) return;
  try {
    await invoke("open_folder", { path: savePath });
  } catch (err) {
    console.error("Erreur ouverture dossier :", err);
  }
}
