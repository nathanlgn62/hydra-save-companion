import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { setRunningGame } from "../stores/gameStore";

export function useWatcher() {
  useEffect(() => {
    // Écoute quand un jeu démarre
    const unlistenStarted = listen<string>("game-started", (event) => {
      setRunningGame(event.payload);
    });

    // Écoute quand le jeu se ferme
    const unlistenClosed = listen<string>("game-closed", () => {
      setRunningGame(null);
    });

    // Nettoyage des listeners au démontage
    return () => {
      unlistenStarted.then((unlisten) => unlisten());
      unlistenClosed.then((unlisten) => unlisten());
    };
  }, []);
}
