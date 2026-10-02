import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { setRunningGame } from "../stores/gameStore";
import { useToasts } from "../stores/toastStore";

export function useWatcher() {
  const { showToast } = useToasts();

  useEffect(() => {
    // Écoute quand un jeu démarre
    const unlistenStarted = listen<string>("game-started", (event) => {
      setRunningGame(event.payload);
      showToast(`Le jeu "${event.payload}" a démarré.`, "info");
    });

    // Écoute quand le jeu se ferme
    const unlistenClosed = listen<string>("game-closed", () => {
      setRunningGame(null);
      showToast("Le jeu a été fermé.", "info");
    });

    // Nettoyage des listeners au démontage
    return () => {
      unlistenStarted.then((unlisten) => unlisten());
      unlistenClosed.then((unlisten) => unlisten());
    };
  }, []);
}
