import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { setGameDownloading } from "../stores/gameStore";
import { HydraGame } from "../types/game";
import { useDownloadGame } from "./useGames";
import { useSettings } from "./useSettings";

export function useAutoDownloadSaves() {
  const queryClient = useQueryClient();
  const downloadMutation = useDownloadGame();
  const { settings } = useSettings();
  const [timeLeft, setTimeLeft] = useState<number | null>(null);

  const intervalSetting = settings.downloadInterval;
  const isEnabled =
    intervalSetting !== "manually" && typeof intervalSetting === "number";

  const downloadMutationRef = useRef(downloadMutation);
  downloadMutationRef.current = downloadMutation;

  const queryClientRef = useRef(queryClient);
  queryClientRef.current = queryClient;

  useEffect(() => {
    if (!isEnabled || typeof intervalSetting !== "number") {
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
      console.log("Lancement de l'autoSync..");
      setTimeLeft(totalSeconds);

      try {
        const games = queryClientRef.current.getQueryData<HydraGame[]>([
          "games",
        ]);

        if (!games || games.length === 0) return;

        const storedToken = localStorage.getItem("gdrive_token");
        if (!storedToken) return;

        const accessToken = storedToken.split("|")[0];

        for (const game of games) {
          if (!game.savePath || !game.localPathExists) continue;

          try {
            const syncStatus = await invoke<any>("check_game_sync_status", {
              token: accessToken,
              gameTitle: game.title,
              savePath: game.savePath,
            });

            if (syncStatus.status === "CloudNewer") {
              console.log(`[AutoSync] Téléchargement pour ${game.title}`);

              // Active l'état de chargement sur la GameCard correspondante
              setGameDownloading(game.title, true);
              try {
                await downloadMutationRef.current.mutateAsync({
                  gameTitle: game.title,
                  savePath: game.savePath,
                });
              } finally {
                // Désactive l'état de chargement quoiqu'il arrive
                setGameDownloading(game.title, false);
              }
            }
          } catch (err) {
            console.error(`[AutoSync] Erreur pour ${game.title}:`, err);
          }
        }
      } catch (err) {
        console.error("[AutoSync] Erreur générale :", err);
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
