import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useMemo } from "react";
import { useToasts } from "../stores/toast-store";
import {
  GameProcessInfo,
  HydraGame,
  RemoteBackupInfo,
  SaveInfoResponse,
  SyncPayload,
  SyncStatusResult,
} from "../types";
import { parseSaveDate } from "../utils/date";
import { useDesktopNotification } from "./use-desktop-notification";
import { useSettings } from "./use-settings";
import { useCloudStatus } from "../stores/cloud-store";

// Fetch global de tous les jeux
export function useGames() {
  const { settings } = useSettings();
  const isDemo = !!settings?.demoMode;
  const { isDriveConnected, cloudProvider } = useCloudStatus();

  const storedToken =
    localStorage.getItem("cloud_token") ||
    localStorage.getItem("gdrive_token") ||
    "";

  return useQuery({
    queryKey: ["games", isDemo, isDriveConnected, cloudProvider, storedToken],
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

      const gamesWithSaves = await Promise.all(
        installedGames.map(async (game) => {
          // On appelle Rust pour récupérer les sauvegardes et la jaquette en parallèle
          const [saveInfo, coverUrl] = await Promise.all([
            invoke<SaveInfoResponse>("get_game_save_info", {
              appId: game.objectId ?? null,
              title: game.title,
              executablePath: game.executablePath ?? null,
            }).catch(() => null),
            invoke<string>("get_steam_cover", {
              appId: game.objectId ? Number(game.objectId) : null,
              title: game.title,
            }).catch(() => ""),
          ]);

      let remoteDate = "Never";
          let backups: RemoteBackupInfo[] = [];
          if (isDriveConnected && storedToken && saveInfo?.resolvedPath) {
            try {
              const provider =
                localStorage.getItem("cloud_provider") || "google-drive";
              const syncStatus = await invoke<SyncStatusResult>("check_game_sync_status", {
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
              remoteDate = "Never";
            }
          }

          return {
            ...game,
            lastLocalSave: saveInfo?.lastModified ?? "Never",
            lastRemoteSave: remoteDate,
            savePath: saveInfo?.resolvedPath ?? null,
            localPathExists: saveInfo?.localPathExists,
            ludasaviPathExists: saveInfo?.ludasaviPathExists,
            cover: (coverUrl && coverUrl.trim().length > 0) ? coverUrl : game.iconUrl,
            backups,
          } as HydraGame;
        }),
      );

      // Notification to Tauri backend
      const monitoredPayload: GameProcessInfo[] = gamesWithSaves.map((game) => ({
        title: game.title,
        executable_name:
          (game.executablePath || game.title).split(/[/\\]/).pop() ||
          game.title,
        save_path: game.savePath ?? null,
      }));
      await invoke("set_monitored_games", { games: monitoredPayload });

      return gamesWithSaves;
    },
  });
}

// Mutation to upload ONE game save
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

      if (!storedToken) {
        showToast("Not connected to a Cloud service", "error");
        desktopNotification(
          "Automatic Synchronization",
          "Not connected to a Cloud service",
        );
        return;
      }

      showToast(
        `Uploading save for "${gameTitle}" in background...`,
        "info",
      );
      desktopNotification(
        `${gameTitle}`,
        `Uploading save to cloud...`,
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
        "Automatic Synchronization",
        "Save upload completed.",
      );
      showToast("Save upload completed.", "success");
      queryClient.refetchQueries({ queryKey: ["games"] });
      queryClient.invalidateQueries({ queryKey: ["cloud-storage"] });
    },
    onError: () => {
      desktopNotification(
        "Automatic Synchronization",
        "Error uploading save",
      );
      showToast("Error uploading save", "error");
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

      if (!storedToken) {
        showToast("Not connected to a Cloud service", "error");
        await desktopNotification(
          "Automatic Synchronization",
          "Not connected to a Cloud service",
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
