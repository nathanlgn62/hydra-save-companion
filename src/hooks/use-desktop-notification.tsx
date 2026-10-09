import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { useCallback } from "react";
import { useTauriWindowFocus } from "./use-app-is-focused";
import { useSettings } from "./use-settings";

export function useDesktopNotification() {
  const { settings } = useSettings();
  const isFocused = useTauriWindowFocus();

  const desktopNotification = useCallback(
    async (title: string, body: string) => {
      try {
        let permissionGranted = await isPermissionGranted();

        if (!permissionGranted) {
          const permission = await requestPermission();
          permissionGranted = permission === "granted";
        }

        if (
          permissionGranted &&
          settings.desktopNotificationsEnabled &&
          !isFocused
        ) {
          sendNotification({ title, body });
        }
      } catch (error) {
        console.error(
          "Erreur lors de l'envoi de la notification native :",
          error,
        );
      }
    },
    [isFocused, settings.desktopNotificationsEnabled],
  );

  return { desktopNotification };
}
