import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
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

  // 1. Requête unique et partagée pour lire les settings
  const { data: settings = DEFAULT_SETTINGS, isLoading: loading } = useQuery({
    queryKey: ["settings"],
    queryFn: async (): Promise<UserSettings> => {
      const store = await getStore();
      const saved = await store.get<UserSettings>("user_prefs");

      if (saved) {
        return { ...DEFAULT_SETTINGS, ...saved };
      }

      await store.set("user_prefs", DEFAULT_SETTINGS);
      await store.save();
      return DEFAULT_SETTINGS;
    },
    staleTime: Infinity, // Garde les settings en mémoire sans refetch inutile
  });

  // 2. Mutation réactive pour sauvegarder et notifier toute l'UI
  const mutation = useMutation({
    mutationFn: async (newPartialSettings: Partial<UserSettings>) => {
      const currentSettings =
        queryClient.getQueryData<UserSettings>(["settings"]) ??
        DEFAULT_SETTINGS;
      const updatedSettings: UserSettings = {
        ...currentSettings,
        ...newPartialSettings,
      };

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
