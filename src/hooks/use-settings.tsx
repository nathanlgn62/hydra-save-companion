import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Store } from "@tauri-apps/plugin-store";
import { UserSettings } from "../types";

export type { UserSettings };

const DEFAULT_SETTINGS: UserSettings = {
  uploadInterval: "afterGameClose",
  downloadInterval: 5,
  autoStartWithSystem: true,
  desktopNotificationsEnabled: true,
  demoMode: false,
};

let storeInstance: Store | null = null;

async function getStore() {
  if (!storeInstance) {
    storeInstance = await Store.load("settings.json");
  }
  return storeInstance;
}

export function useSettings() {
  const queryClient = useQueryClient();

  // 1. Shared query to read settings and ensure OS autostart matches user preference
  const { data: settings = DEFAULT_SETTINGS, isLoading: loading } = useQuery({
    queryKey: ["settings"],
    queryFn: async (): Promise<UserSettings> => {
      const store = await getStore();
      const saved = await store.get<UserSettings>("user_prefs");

      let currentSettings = DEFAULT_SETTINGS;
      if (saved) {
        currentSettings = { ...DEFAULT_SETTINGS, ...saved };
      } else {
        await store.set("user_prefs", DEFAULT_SETTINGS);
        await store.save();
      }

      // In production builds, check real OS autostart state and sync if needed.
      // In development (dev mode), skip to avoid registering debug binary with devUrl (localhost:1420).
      if (import.meta.env.PROD) {
        try {
          const osAutostartEnabled = await isEnabled();
          if (currentSettings.autoStartWithSystem && !osAutostartEnabled) {
            await enable();
          } else if (!currentSettings.autoStartWithSystem && osAutostartEnabled) {
            await disable();
          }
        } catch (err) {
          console.warn("Failed to query or apply OS autostart state:", err);
        }
      }

      return currentSettings;
    },
    staleTime: Infinity,
  });

  // 2. Reactive mutation to update settings and apply OS changes
  const mutation = useMutation({
    mutationFn: async (newPartialSettings: Partial<UserSettings>) => {
      const currentSettings =
        queryClient.getQueryData<UserSettings>(["settings"]) ??
        DEFAULT_SETTINGS;
      const updatedSettings: UserSettings = {
        ...currentSettings,
        ...newPartialSettings,
      };

      // If autoStartWithSystem changed, apply to OS in production builds
      if (
        import.meta.env.PROD &&
        newPartialSettings.autoStartWithSystem !== undefined &&
        newPartialSettings.autoStartWithSystem !== currentSettings.autoStartWithSystem
      ) {
        try {
          if (newPartialSettings.autoStartWithSystem) {
            await enable();
          } else {
            await disable();
          }
        } catch (err) {
          console.error("Failed to update OS autostart:", err);
        }
      }

      const store = await getStore();
      await store.set("user_prefs", updatedSettings);
      await store.save();

      return updatedSettings;
    },
    onSuccess: (updatedSettings) => {
      // Invalide et met à jour instantanément toutes les instances de GameCard
      queryClient.setQueryData(["settings"], updatedSettings);
    },
  });

  return {
    settings,
    updateSettings: mutation.mutateAsync,
    loading,
  };
}
