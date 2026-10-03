import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { useCallback } from "react";
import { useSettings } from "./useSettings";

export function useDesktopNotification() {
  const { settings } = useSettings();

  const desktopNotification = useCallback(
    async (title: string, body: string) => {
      try {
        let permissionGranted = await isPermissionGranted();

        if (!permissionGranted) {
          const permission = await requestPermission();
          permissionGranted = permission === "granted";
        }

        if (permissionGranted && settings.desktopNotificationsEnabled) {
          sendNotification({ title, body });
        }
      } catch (error) {
        console.error(
          "Erreur lors de l'envoi de la notification native :",
          error,
        );
      }
    },
    [],
  );

  return { desktopNotification };
}
