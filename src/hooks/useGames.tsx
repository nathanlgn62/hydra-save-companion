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
      const accessToken = storedToken ? storedToken.split("|")[0] : null;

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
            // On vérifie le cloud si on a un token, même si le dossier local n'existe pas encore
            if (accessToken) {
              try {
                const syncStatus = await invoke<any>("check_game_sync_status", {
                  token: accessToken,
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
export function useSyncGame() {
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
      const accessToken = storedToken.split("|")[0];

      return await invoke<string>("upload_game_save_to_drive", {
        token: accessToken,
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

export function useDownloadGame() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ gameTitle, savePath }: SyncPayload) => {
      const storedToken = localStorage.getItem("gdrive_token");
      if (!storedToken) throw new Error("Non connecté à Google Drive");
      const accessToken = storedToken.split("|")[0];

      // Appel de la commande Rust Tauri 'download_game_save'
      const response = await invoke<string>("download_game_save_from_drive", {
        token: accessToken,
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
