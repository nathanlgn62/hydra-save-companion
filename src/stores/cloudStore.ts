import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type CloudListener = (isConnected: boolean) => void;

let isDriveConnected = !!localStorage.getItem("gdrive_token");
let isConnecting = false;
const listeners = new Set<CloudListener>();

const notifyListeners = () => {
  listeners.forEach((listener) => listener(isDriveConnected));
};

export const loginGoogle = async () => {
  isConnecting = true;
  try {
    const token = await invoke<string>("login_google");
    if (token) {
      localStorage.setItem("gdrive_token", token);
      isDriveConnected = true;
      notifyListeners();
    }
  } catch (err) {
    console.error("Erreur d'authentification Google :", err);
    alert("Échec de la connexion Google.");
  } finally {
    isConnecting = false;
  }
};

export const disconnectGoogle = () => {
  localStorage.removeItem("gdrive_token");
  isDriveConnected = false;
  notifyListeners();
};

export const getDriveStatus = () => isDriveConnected;

export const subscribeToCloudStatus = (listener: CloudListener) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

export function useCloudStatus() {
  const [connected, setConnected] = useState(getDriveStatus());

  useEffect(() => {
    return subscribeToCloudStatus((status) => {
      setConnected(status);
    });
  }, []);

  return { isDriveConnected: connected, loginGoogle, disconnectGoogle };
}
