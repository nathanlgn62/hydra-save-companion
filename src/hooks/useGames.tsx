import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
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

interface GameClosedPayload {
  title: string;
  save_path: string | null;
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

  return useMutation({
    mutationFn: async ({
      gameTitle,
      savePath,
    }: {
      gameTitle: string;
      savePath: string;
    }) => {
      const storedToken = localStorage.getItem("gdrive_token");
      if (!storedToken) throw new Error("Non connecté à Google Drive");

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

  return useMutation({
    mutationFn: async ({ gameTitle, savePath }: SyncPayload) => {
      const storedToken = localStorage.getItem("gdrive_token");
      if (!storedToken) throw new Error("Non connecté à Google Drive");

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

export function useAutoUploadCountdown(
  gameTitle: string,
  savePath: string | null,
  uploadInterval: string,
  queryClient: any,
) {
  const [countdown, setCountdown] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const syncMutation = useUploadSave();
  const syncMutationRef = useRef(syncMutation);

  useEffect(() => {
    syncMutationRef.current = syncMutation;
  }, [syncMutation]);

  const clearCountdown = () => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    setCountdown(null);
  };

  const startCountdown = (duration = 45) => {
    clearCountdown();
    if (!savePath) return;

    setCountdown(duration);
    timerRef.current = setInterval(() => {
      setCountdown((prev) => {
        if (prev === null || prev <= 1) {
          clearCountdown();
          syncMutationRef.current.mutate({ gameTitle, savePath });
          return null;
        }
        return prev - 1;
      });
    }, 1000);
  };

  useEffect(() => {
    let unlistenStarted: (() => void) | null = null;
    let unlistenClosed: (() => void) | null = null;

    const setup = async () => {
      // Si le jeu démarre, on annule le compte à rebours en cours
      unlistenStarted = await listen<string>("game-started", (event) => {
        if (event.payload === gameTitle) {
          clearCountdown();
        }
      });

      // Si le jeu se ferme
      unlistenClosed = await listen<GameClosedPayload>(
        "game-closed",
        (event) => {
          if (event.payload.title !== gameTitle || !savePath) return;

          if (uploadInterval === "afterGameClose") {
            startCountdown(45);
          } else {
            queryClient.invalidateQueries({ queryKey: ["games"] });
          }
        },
      );
    };

    setup();

    return () => {
      unlistenStarted?.();
      unlistenClosed?.();
      clearCountdown();
    };
  }, [gameTitle, savePath, uploadInterval, queryClient]);

  return { countdown, clearCountdown };
}

import { useMemo } from "react";
import { parseSaveDate } from "../utils/date";

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
