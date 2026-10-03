import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";

export function useTauriWindowFocus() {
  const [isFocused, setIsFocused] = useState(true);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setup = async () => {
      const appWindow = getCurrentWindow();

      // On écoute directement les changements de focus de la fenêtre Tauri
      unlisten = await appWindow.onFocusChanged(({ payload }) => {
        setIsFocused(payload);
      });
    };

    setup();

    return () => {
      unlisten?.();
    };
  }, []);

  return isFocused;
}
