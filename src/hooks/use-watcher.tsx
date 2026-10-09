import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { setRunningGame } from "../stores/game-store";
import { useToasts } from "../stores/toast-store";
import { GameClosedPayload } from "../types";
import { useDesktopNotification } from "./use-desktop-notification";
import { useUploadSave } from "./use-games";
import { useSettings } from "./use-settings";

export function useWatcher() {
  const queryClient = useQueryClient();
  const { showToast } = useToasts();
  const { desktopNotification } = useDesktopNotification();
  const { settings } = useSettings();
  const uploadMutation = useUploadSave();

  const [countdown, setCountdown] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const lastEventRef = useRef<{ type: string; time: number }>({
    type: "",
    time: 0,
  });

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

        const now = Date.now();
        if (
          lastEventRef.current.type === `started-${gameTitle}` &&
          now - lastEventRef.current.time < 1000
        ) {
          return;
        }
        lastEventRef.current = { type: `started-${gameTitle}`, time: now };

        setRunningGame(gameTitle);
        clearCountdown();
        showToast(`Le jeu "${gameTitle}" a démarré.`, "info");
        desktopNotification(`${gameTitle}`, `Le jeu a démarré.`);
      });

      unlistenClosed = await listen<GameClosedPayload>(
        "game-closed",
        async (event) => {
          const gameTitle = event.payload.title;

          const now = Date.now();
          if (
            lastEventRef.current.type === `closed-${gameTitle}` &&
            now - lastEventRef.current.time < 1000
          ) {
            return;
          }
          lastEventRef.current = { type: `closed-${gameTitle}`, time: now };

          setRunningGame(null);

          // 1. On affiche immédiatement le toast et la notif de fermeture
          if (
            settings.uploadInterval === "afterGameClose" &&
            event.payload.savePath
          ) {
            startCountdown(gameTitle, event.payload.savePath, 45);
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

          // 2. On rafraîchit les requêtes en arrière-plan sans bloquer l'UI
          queryClient.invalidateQueries({ queryKey: ["games"] });
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
