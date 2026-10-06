import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { getStoredStatus } from "../utils/storage";

type CloudListener = (status: {
  isConnected: boolean;
  provider: string | null;
}) => void;

const listeners = new Set<CloudListener>();

const notifyListeners = () => {
  const status = getStoredStatus();
  listeners.forEach((listener) => listener(status));
};

export const loginCloud = async (provider: string) => {
  try {
    const tokenResponse = await invoke<string>("login_cloud", { provider });

    if (tokenResponse) {
      localStorage.setItem("cloud_provider", provider);
      localStorage.setItem("cloud_token", tokenResponse);
      notifyListeners();
    }
  } catch (err) {
    console.error(`Erreur d'authentification pour ${provider} :`, err);
    alert(`Échec de la connexion à ${provider} : ${err}`);
  }
};

export const disconnectCloud = () => {
  localStorage.removeItem("cloud_token");
  localStorage.removeItem("cloud_provider");
  localStorage.removeItem("gdrive_token");

  notifyListeners();
};

export const getCloudStatus = () => getStoredStatus();

export const subscribeToCloudStatus = (listener: CloudListener) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

export function useCloudStatus() {
  const [status, setStatus] = useState(getCloudStatus());

  useEffect(() => {
    setStatus(getCloudStatus());

    return subscribeToCloudStatus((newStatus) => {
      setStatus(newStatus);
    });
  }, []);

  return {
    isDriveConnected: status.isConnected,
    cloudProvider: status.provider,
    loginCloud,
    disconnectCloud,
    loginGoogle: () => loginCloud("google-drive"),
    disconnectGoogle: disconnectCloud,
  };
}
