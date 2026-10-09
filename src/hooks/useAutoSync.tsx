// [AUTOSYNC DOWNLOAD DÉSACTIVÉ]
// Les imports et la logique d'auto-sync download sont commentés ci-dessous pour être réactivés facilement plus tard.
/*
import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { setGameDownloading } from "../stores/gameStore";
import { useToasts } from "../stores/toastStore";
import { HydraGame, SyncStatusResult } from "../types";
import { useDesktopNotification } from "./useDesktopNotification";
import { useDownloadSave, useGames } from "./useGames";
import { useSettings } from "./useSettings";
*/

export function useAutoDownloadSaves() {
  // Mode automatique de téléchargement désactivé
  return {
    timeLeft: null,
    isManual: true,
  };
}

/*
export function useAutoDownloadSavesActive() {
  const queryClient = useQueryClient();
  const downloadMutation = useDownloadSave();
  const { settings } = useSettings();
  const [timeLeft, setTimeLeft] = useState<number | null>(null);
  const { showToast } = useToasts();
  const { desktopNotification } = useDesktopNotification();
  const { data: games } = useGames();

  const intervalSetting = settings.downloadInterval;
  const isEnabled =
    intervalSetting !== "manually" && typeof intervalSetting === "number";

  // Références pour éviter les closures périmées sans redéclencher l'effet
  const downloadMutationRef = useRef(downloadMutation);
  downloadMutationRef.current = downloadMutation;

  const queryClientRef = useRef(queryClient);
  queryClientRef.current = queryClient;

  const showToastRef = useRef(showToast);
  showToastRef.current = showToast;

  // Verrou pour empêcher les lancements simultanés si une synchro prend du temps
  const isSyncingRef = useRef(false);

  useEffect(() => {
    if (
      !isEnabled ||
      typeof intervalSetting !== "number" ||
      !games ||
      games.length === 0
    ) {
      setTimeLeft(null);
      return;
    }

    const totalSeconds = intervalSetting * 60;
    setTimeLeft(totalSeconds);

    const countdownTimer = setInterval(() => {
      setTimeLeft((prev) => {
        if (prev === null || prev <= 1) return totalSeconds;
        return prev - 1;
      });
    }, 1000);

    const syncTimer = setInterval(async () => {
      // Si une synchro est déjà en cours, on ignore cette itération
      if (isSyncingRef.current) {
        console.log(
          "[AutoSync] Une synchronisation est déjà en cours, ignorée.",
        );
        return;
      }

      isSyncingRef.current = true;
      console.log("Lancement de l'autoSync..");
      setTimeLeft(totalSeconds);

      try {
        const games = queryClientRef.current.getQueryData<HydraGame[]>([
          "games",
        ]);

        if (!games || games.length === 0) {
          showToastRef.current(
            "Aucun jeu trouvé pour la synchronisation.",
            "info",
            3000,
          );

          await desktopNotification(
            "Synchronisation automatique",
            "Aucun jeu trouvé pour la synchronisation.",
          );

          return;
        }

        const storedToken =
          localStorage.getItem("cloud_token") ||
          localStorage.getItem("gdrive_token");
        if (!storedToken) return;

        for (const game of games) {
          if (!game.savePath || !game.localPathExists) continue;

          try {
            const provider =
              localStorage.getItem("cloud_provider") || "google-drive";
            const syncStatus = await invoke<SyncStatusResult>(
              "check_game_sync_status",
              {
                token: storedToken,
                gameTitle: game.title,
                savePath: game.savePath,
                provider,
              },
            );

            if (syncStatus.status === "CloudNewer") {
              console.log(`[AutoSync] Téléchargement pour ${game.title}`);
              showToastRef.current(
                `Téléchargement de la sauvegarde pour ${game.title}...`,
                "info",
                3000,
              );
              await desktopNotification(
                "Synchronisation automatique",
                `Téléchargement de la sauvegarde pour ${game.title}...`,
              );
              setGameDownloading(game.title, true);
              try {
                await downloadMutationRef.current.mutateAsync({
                  gameTitle: game.title,
                  savePath: game.savePath,
                });
              } finally {
                setGameDownloading(game.title, false);
              }
            }
          } catch (err) {
            showToastRef.current(
              `Erreur lors de la synchronisation pour ${game.title}.`,
              "error",
              5000,
            );
            desktopNotification(
              "Synchronisation automatique",
              `Erreur lors de la synchronisation pour ${game.title}.`,
            );
            console.error(`[AutoSync] Erreur pour ${game.title}:`, err);
          }
        }
      } catch (err) {
        showToastRef.current(
          "Erreur générale lors de la synchronisation automatique.",
          "error",
          5000,
        );
        desktopNotification(
          "Synchronisation automatique",
          "Erreur générale lors de la synchronisation automatique.",
        );
        console.error("[AutoSync] Erreur générale :", err);
      } finally {
        // Libération du verrou une fois terminé
        isSyncingRef.current = false;
      }
    }, totalSeconds * 1000);

    return () => {
      clearInterval(countdownTimer);
      clearInterval(syncTimer);
    };
  }, [isEnabled, intervalSetting]);

  return {
    timeLeft: timeLeft !== null ? formatTime(timeLeft) : null,
    isManual: !isEnabled,
  };
}

function formatTime(totalSeconds: number): string {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
}
*/
