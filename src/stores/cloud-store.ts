import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { getStoredStatus } from "../utils/storage";
import { CloudStatus } from "../types";

import { queryClient } from "../utils/query-client";
import { HydraGame } from "../types";

type CloudListener = (status: CloudStatus) => void;

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

      // Invalide le cache React Query pour charger les données cloud fraîches
      queryClient.invalidateQueries({ queryKey: ["games"] });
      queryClient.invalidateQueries({ queryKey: ["cloud-storage"] });

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

  // Réinitialise immédiatement en mémoire les informations distantes des jeux
  queryClient.setQueriesData<HydraGame[]>({ queryKey: ["games"] }, (oldGames) => {
    if (!oldGames) return oldGames;
    return oldGames.map((game) => ({
      ...game,
      lastRemoteSave: "Jamais",
      backups: [],
    }));
  });

  // Invalide les requêtes pour assurer une synchronisation propre
  queryClient.invalidateQueries({ queryKey: ["games"] });
  queryClient.invalidateQueries({ queryKey: ["cloud-storage"] });

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
