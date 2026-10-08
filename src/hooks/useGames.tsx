import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useMemo } from "react";
import { useToasts } from "../stores/toastStore";
import { HydraGame } from "../types/game";
import { parseSaveDate } from "../utils/date";
import { useDesktopNotification } from "./useDesktopNotification";

import { useSettings } from "./useSettings";

interface SaveInfoResponse {
  ludasaviPathExists: boolean;
  localPathExists: boolean;
  resolvedPath: string | null;
  lastModified: string | null;
}

interface SyncPayload {
  gameTitle: string;
  savePath: string;
  fileId?: string;
}

// Fetch global de tous les jeux
export function useGames() {
  const { settings } = useSettings();
  const isDemo = !!settings?.demoMode;

  return useQuery({
    queryKey: ["games", isDemo],
    queryFn: async () => {
      let installedGames: HydraGame[] = [];
      if (isDemo) {
        installedGames = await invoke<HydraGame[]>("setup_demo_environment");
      } else {
        try {
          installedGames = await invoke<HydraGame[]>("get_installed_games");
        } catch (e) {
          console.warn("Hydra games not detected, checking demo mode or empty:", e);
          installedGames = [];
        }
      }

      const storedToken =
        localStorage.getItem("cloud_token") ||
        localStorage.getItem("gdrive_token");

      const gamesWithSaves = await Promise.all(
        installedGames.map(async (game) => {
          // On appelle Rust pour récupérer les sauvegardes et la jaquette en parallèle
          const [saveInfo, coverUrl] = await Promise.all([
            invoke<SaveInfoResponse>("get_game_save_info", {
              appId: game.objectId ?? null,
              title: game.title,
            }).catch(() => null),
            invoke<string>("get_steam_cover", {
              appId: game.objectId ? Number(game.objectId) : null,
            }).catch(
              () =>
                `https://cdn.cloudflare.steamstatic.com/steam/apps/${game.objectId}/library_600x900_2x.jpg`,
            ),
          ]);

          console.log("saveInfo", saveInfo);

          let remoteDate = "Jamais";
          let backups: any[] = [];
          if (storedToken && saveInfo?.resolvedPath) {
            try {
              const provider =
                localStorage.getItem("cloud_provider") || "google-drive";
              const syncStatus = await invoke<any>("check_game_sync_status", {
                token: storedToken,
                gameTitle: game.title,
                savePath: saveInfo.resolvedPath,
                provider,
              });

              if (Array.isArray(syncStatus.backups)) {
                backups = syncStatus.backups;
              }

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
            lastLocalSave: saveInfo?.lastModified ?? "Jamais",
            lastRemoteSave: remoteDate,
            savePath: saveInfo?.resolvedPath,
            localPathExists: saveInfo?.localPathExists,
            ludasaviPathExists: saveInfo?.ludasaviPathExists,
            cover: coverUrl,
            backups,
          } as HydraGame;
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
      const storedToken =
        localStorage.getItem("cloud_token") ||
        localStorage.getItem("gdrive_token");
      const provider = localStorage.getItem("cloud_provider") || "google-drive";

      if (!storedToken && provider !== "proton-drive") {
        showToast("Non connecté à un service Cloud", "error");
        desktopNotification(
          "Synchronisation automatique",
          "Non connecté à un service Cloud",
        );
        return;
      }

      showToast(
        `Upload automatique de la sauvegarde de "${gameTitle}" en cours...`,
        "info",
      );
      desktopNotification(
        `${gameTitle}`,
        `Upload automatique de la sauvegarde en cours...`,
      );

      return await invoke<string>("upload_game_save_to_drive", {
        token: storedToken || "",
        gameTitle,
        savePath,
        provider,
      });
    },
    onSuccess: () => {
      desktopNotification(
        "Synchronisation automatique",
        "Upload de la sauvegarde terminé.",
      );
      showToast("Upload de la sauvegarde terminé.", "success");
      queryClient.refetchQueries({ queryKey: ["games"] });
    },
    onError: () => {
      desktopNotification(
        "Synchronisation automatique",
        `Erreur lors de l'upload de la sauvegarde`,
      );
      showToast("Erreur lors de l'upload de la sauvegarde", "error");
    },
  });
}

export function useDownloadSave() {
  const queryClient = useQueryClient();
  const { showToast } = useToasts();
  const { desktopNotification } = useDesktopNotification();

  return useMutation({
    mutationFn: async ({ gameTitle, savePath, fileId }: SyncPayload) => {
      const storedToken =
        localStorage.getItem("cloud_token") ||
        localStorage.getItem("gdrive_token");
      const provider = localStorage.getItem("cloud_provider") || "google-drive";

      if (!storedToken && provider !== "proton-drive") {
        showToast("Non connecté à un service Cloud", "error");
        await desktopNotification(
          "Synchronisation automatique",
          "Non connecté à un service Cloud",
        );
        return;
      }

      // Appel de la commande Rust Tauri 'download_game_save_from_drive'
      const response = await invoke<string>("download_game_save_from_drive", {
        token: storedToken || "",
        gameTitle,
        savePath,
        provider,
        fileId: fileId ?? null,
      });

      return response;
    },
    onSuccess: () => {
      // Rafraîchit les données des jeux (lastLocalSave, badges, etc.)
      queryClient.invalidateQueries({ queryKey: ["games"] });
    },
  });
}

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
