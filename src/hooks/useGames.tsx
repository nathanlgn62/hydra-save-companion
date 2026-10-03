import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { HydraGame } from "../types/game";

interface SaveInfoResponse {
  ludasaviPathExists: boolean;
  localPathExists: boolean;
  resolvedPath: string | null;
  lastModified: string | null;
}

interface SyncPayload {
  gameTitle: string;
  savePath: string;
}

// Fetch global de tous les jeux
export function useGames() {
  return useQuery({
    queryKey: ["games"],
    queryFn: async () => {
      const installedGames = await invoke<HydraGame[]>("get_installed_games");
      const storedToken = localStorage.getItem("gdrive_token");

      const gamesWithSaves = await Promise.all(
        installedGames.map(async (game) => {
          try {
            const saveInfo = await invoke<SaveInfoResponse>(
              "get_game_save_info",
              {
                appId: game.objectId ?? null,
                title: game.title,
              },
            );

            let remoteDate = "Jamais";
            if (storedToken) {
              try {
                const syncStatus = await invoke<any>("check_game_sync_status", {
                  token: storedToken, // <-- On passe le token complet
                  gameTitle: game.title,
                  savePath: saveInfo.resolvedPath ?? "",
                });

                if (syncStatus.status === "UpToDate")
                  remoteDate = syncStatus.localTime;
                else if (
                  ["LocalNewer", "CloudNewer", "CloudOnly"].includes(
                    syncStatus.status,
                  )
                )
                  remoteDate = syncStatus.cloudTime;
              } catch {
                remoteDate = "Jamais";
              }
            }

            return {
              ...game,
              lastLocalSave: saveInfo.lastModified ?? "Jamais",
              lastRemoteSave: remoteDate,
              savePath: saveInfo.resolvedPath,
              localPathExists: saveInfo.localPathExists,
              ludasaviPathExists: saveInfo.ludasaviPathExists,
            } as HydraGame;
          } catch {
            return game;
          }
        }),
      );

      // Notification au backend Tauri
      const monitoredPayload = gamesWithSaves.map((game) => ({
        title: game.title,
        executable_name:
          (game.executablePath || game.title).split(/[/\\]/).pop() ||
          game.title,
        save_path: game.savePath ?? null,
      }));
      await invoke("set_monitored_games", { games: monitoredPayload });

      console.log(gamesWithSaves);

      return gamesWithSaves;
    },
  });
}

// Mutation pour la synchronisation d'UN jeu
export function useUploadSave() {
  const queryClient = useQueryClient();

  const { showToast } = useToasts();
  const { desktopNotification } = useDesktopNotification();

  return useMutation({
    mutationFn: async ({
      gameTitle,
      savePath,
    }: {
      gameTitle: string;
      savePath: string;
    }) => {
      const storedToken = localStorage.getItem("gdrive_token");
      if (!storedToken) {
        showToast("Non connecté à Google Drive", "error");
        desktopNotification(
          "Synchronisation automatique",
          "Non connecté à Google Drive",
        );
        return;
      }

      return await invoke<string>("upload_game_save_to_drive", {
        token: storedToken,
        gameTitle,
        savePath,
      });
    },
    onSuccess: () => {
      // Invalide le cache des jeux -> déclenche un re-fetch silencieux en arrière-plan
      queryClient.invalidateQueries({ queryKey: ["games"] });
    },
  });
}

export function useDownloadSave() {
  const queryClient = useQueryClient();
  const { showToast } = useToasts();
  const { desktopNotification } = useDesktopNotification();

  return useMutation({
    mutationFn: async ({ gameTitle, savePath }: SyncPayload) => {
      const storedToken = localStorage.getItem("gdrive_token");
      if (!storedToken) {
        showToast("Non connecté à Google Drive", "error");
        await desktopNotification(
          "Synchronisation automatique",
          "Non connecté à Google Drive",
        );
        return;
      }

      // Appel de la commande Rust Tauri 'download_game_save'
      const response = await invoke<string>("download_game_save_from_drive", {
        token: storedToken,
        gameTitle,
        savePath,
      });

      return response;
    },
    onSuccess: () => {
      // Rafraîchit les données des jeux (lastLocalSave, badges, etc.)
      queryClient.invalidateQueries({ queryKey: ["games"] });
    },
  });
}

import { useMemo } from "react";
import { useToasts } from "../stores/toastStore";
import { parseSaveDate } from "../utils/date";
import { useDesktopNotification } from "./useDesktopNotification";

export function useGameSyncStatus(
  lastLocalSave?: string | null,
  lastRemoteSave?: string | null,
) {
  return useMemo(() => {
    const localTime = parseSaveDate(lastLocalSave);
    const remoteTime = parseSaveDate(lastRemoteSave);

    if (!remoteTime && !localTime) return "none";
    if (!localTime && remoteTime) return "down";
    if (localTime && !remoteTime) return "up";

    if (remoteTime! > localTime!) return "down";
    if (localTime! > remoteTime!) return "up";
    return "synced";
  }, [lastLocalSave, lastRemoteSave]);
}
