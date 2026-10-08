import { X } from "lucide-react";
import { useModalAnimation } from "../../hooks/useModalAnimation";
import { UserSettings } from "../../types";
import Toggle from "../UI/toggle";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  settings: UserSettings;
  updateSettings: (newSettings: Partial<UserSettings>) => void;
  loadingSettings: boolean;
}

export default function SettingsModal({
  isOpen,
  onClose,
  settings,
  updateSettings,
  loadingSettings,
}: SettingsModalProps) {
  const { isRendered, isVisible, closeWithAnimation } = useModalAnimation(
    isOpen,
    onClose,
  );

  if (!isRendered) return null;

  return (
    <div
      className={`fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm transition-all duration-200 ease-out ${
        isVisible ? "opacity-100" : "opacity-0 pointer-events-none"
      }`}
      onClick={closeWithAnimation}
    >
      <div
        className={`bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md shadow-2xl overflow-hidden flex flex-col transition-all duration-200 ease-out transform ${
          isVisible
            ? "opacity-100 scale-100 translate-y-0"
            : "opacity-0 scale-95 translate-y-3"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800">
          <h2 className="text-base font-semibold text-slate-100">Paramètres</h2>
          <button
            type="button"
            onClick={closeWithAnimation}
            className="p-1 text-slate-400 hover:text-white rounded-lg transition cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4 flex-1 overflow-y-auto">
          {loadingSettings ? (
            <div className="text-center py-4 text-xs text-slate-400">
              Chargement des paramètres...
            </div>
          ) : (
            <>
              <div>
                <label className="text-xs font-medium text-slate-300 block mb-1.5">
                  Fréquence de téléchargement des sauvegardes cloud
                </label>
                <select
                  value={settings.downloadInterval ?? "manually"}
                  onChange={(e) =>
                    updateSettings({
                      downloadInterval:
                        e.target.value === "manually"
                          ? "manually"
                          : Number(e.target.value),
                    })
                  }
                  className="w-full bg-slate-800 border border-slate-700 rounded-xl px-3 py-2 text-sm text-slate-200 outline-none focus:border-indigo-500 transition cursor-pointer"
                >
                  <option value="5">Toutes les 5 minutes</option>
                  <option value="15">Toutes les 15 minutes</option>
                  <option value="30">Toutes les 30 minutes</option>
                  <option value="manually">Manuel uniquement</option>
                </select>
              </div>

              <div>
                <label className="text-xs font-medium text-slate-300 block mb-1.5">
                  Fréquence de téléversement des sauvegardes locales
                </label>
                <select
                  value={settings.uploadInterval}
                  onChange={(e) =>
                    updateSettings({
                      uploadInterval: e.target.value as
                        | "afterGameClose"
                        | "manually",
                    })
                  }
                  className="w-full bg-slate-800 border border-slate-700 rounded-xl px-3 py-2 text-sm text-slate-200 outline-none focus:border-indigo-500 transition cursor-pointer"
                >
                  <option value="afterGameClose">
                    À chaque fermeture d'un jeu
                  </option>
                  <option value="manually">Manuel uniquement</option>
                </select>
              </div>

              <Toggle
                label="Démarrage automatique"
                description="Lancer l'application avec le système"
                checked={settings.autoStartWithSystem}
                onChange={(checked) =>
                  updateSettings({
                    autoStartWithSystem: checked,
                  })
                }
              />

              <Toggle
                label="Notifications de bureau"
                description="Afficher les notifications système Tauri"
                checked={settings.desktopNotificationsEnabled ?? true}
                onChange={(checked) =>
                  updateSettings({
                    desktopNotificationsEnabled: checked,
                  })
                }
              />

              <div className="pt-2 border-t border-slate-800/80">
                <Toggle
                  label="Mode Simulation / Démo"
                  description="Simuler des jeux et des sauvegardes locales pour tester l'application"
                  checked={settings.demoMode ?? false}
                  onChange={(checked) =>
                    updateSettings({
                      demoMode: checked,
                    })
                  }
                />
              </div>
            </>
          )}
        </div>

        <div className="px-6 py-4 border-t border-slate-800 flex justify-end gap-3 bg-slate-950/50">
          <button
            type="button"
            onClick={closeWithAnimation}
            className="px-4 py-2 rounded-xl text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-200 transition cursor-pointer"
          >
            Fermer
          </button>
        </div>
      </div>
    </div>
  );
}
