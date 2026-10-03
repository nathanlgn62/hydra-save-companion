import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { setRunningGame } from "../stores/gameStore";
import { useToasts } from "../stores/toastStore";
import { HydraGame } from "../types/game";
import { useDesktopNotification } from "./useDesktopNotification";
import { useUploadSave } from "./useGames";
import { useSettings } from "./useSettings";

interface GameClosedPayload {
  title: string;
  savePath?: string;
}

export function useWatcher() {
  const queryClient = useQueryClient();
  const { showToast } = useToasts();
  const { desktopNotification } = useDesktopNotification();
  const { settings } = useSettings();
  const uploadMutation = useUploadSave();

  const [countdown, setCountdown] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const uploadMutationRef = useRef(uploadMutation);
  uploadMutationRef.current = uploadMutation;

  const clearCountdown = () => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    setCountdown(null);
  };

  const startCountdown = (
    gameTitle: string,
    savePath: string,
    duration = 45,
  ) => {
    clearCountdown();
    setCountdown(duration);

    timerRef.current = setInterval(() => {
      setCountdown((prev) => {
        if (prev === null || prev <= 1) {
          clearCountdown();
          uploadMutationRef.current.mutate({ gameTitle, savePath });
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
      unlistenStarted = await listen<string>("game-started", (event) => {
        const gameTitle = event.payload;
        setRunningGame(gameTitle);
        clearCountdown();
        showToast(`Le jeu "${gameTitle}" a démarré.`, "info");
        desktopNotification(`${gameTitle}`, `Le jeu a démarré.`);
      });

      unlistenClosed = await listen<GameClosedPayload>(
        "game-closed",
        async (event) => {
          const gameTitle = event.payload.title;
          setRunningGame(null);

          await queryClient.refetchQueries({ queryKey: ["games"] });

          const games = queryClient.getQueryData<HydraGame[]>(["games"]);
          const matchedGame = games?.find((g) => g.title === gameTitle);
          const savePath = matchedGame?.savePath || event.payload.savePath;

          if (settings.uploadInterval === "afterGameClose" && savePath) {
            startCountdown(gameTitle, savePath, 45);
            showToast(
              `Le jeu "${gameTitle}" a été fermé. La sauvegarde sera synchronisée dans 45 secondes.`,
              "info",
            );
            desktopNotification(
              `${gameTitle}`,
              `La sauvegarde sera synchronisée dans 45 secondes.`,
            );
          } else {
            showToast(`Le jeu "${gameTitle}" a été fermé.`, "info");
            desktopNotification(`${gameTitle}`, `Le jeu a été fermé.`);
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
  }, [settings.uploadInterval, queryClient, showToast, desktopNotification]);

  return { countdown, clearCountdown };
}
