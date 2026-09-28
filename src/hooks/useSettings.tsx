import { Store } from "@tauri-apps/plugin-store";
import { useEffect, useState } from "react";

export interface UserSettings {
  uploadInterval: "afterGameClose" | "manually";
  downloadInterval: number | null;
  autoStartWithSystem: boolean;
}

const DEFAULT_SETTINGS: UserSettings = {
  uploadInterval: "afterGameClose",
  downloadInterval: 5,
  autoStartWithSystem: true,
};

let storeInstance: Store | null = null;

async function getStore() {
  if (!storeInstance) {
    storeInstance = await Store.load("settings.json");
  }
  return storeInstance;
}

export function useSettings() {
  const [settings, setSettings] = useState<UserSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function loadSettings() {
      try {
        const store = await getStore();
        const savedSettings = await store.get<UserSettings>("user_prefs");

        if (savedSettings) {
          setSettings({ ...DEFAULT_SETTINGS, ...savedSettings });
        } else {
          await store.set("user_prefs", DEFAULT_SETTINGS);
          await store.save();
        }
      } catch (err) {
        console.error("Erreur chargement settings:", err);
      } finally {
        setLoading(false);
      }
    }

    loadSettings();
  }, []);

  const updateSettings = async (newPartialSettings: Partial<UserSettings>) => {
    try {
      let updatedSettings: UserSettings = DEFAULT_SETTINGS;

      setSettings((prev) => {
        updatedSettings = { ...prev, ...newPartialSettings };
        return updatedSettings;
      });

      const store = await getStore();
      await store.set("user_prefs", updatedSettings);
      await store.save();
    } catch (err) {
      console.error("Erreur sauvegarde settings:", err);
    }
  };

  return { settings, updateSettings, loading };
}