import { invoke } from "@tauri-apps/api/core";
import { Cloud, CloudOff, RefreshCw, Settings, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useAutoDownloadSaves } from "../hooks/useAutoSync";
import { useSettings } from "../hooks/useSettings";

export default function Header() {
  const [driveConnected, setDriveConnected] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);

  const { timeLeft, isManual } = useAutoDownloadSaves();
  const { settings, updateSettings, loading: loadingSettings } = useSettings();

  useEffect(() => {
    const token = localStorage.getItem("gdrive_token");
    if (token) {
      setDriveConnected(true);
    }
  }, []);

  const handleGoogleLogin = async () => {
    setLoading(true);
    try {
      const token = await invoke<string>("login_google");
      if (token) {
        localStorage.setItem("gdrive_token", token);
        setDriveConnected(true);
      }
    } catch (err) {
      console.error("Erreur d'authentification Google :", err);
      alert("Échec de la connexion Google.");
    } finally {
      setLoading(false);
    }
  };

  const handleDisconnect = () => {
    localStorage.removeItem("gdrive_token");
    setDriveConnected(false);
  };

  // Petite fonction utilitaire pour formater l'affichage de l'intervalle dans le badge

  return (
    <>
      <header className="h-14 bg-slate-900 border-b border-slate-800 px-4 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-indigo-600 flex items-center justify-center font-bold text-lg text-white shadow-lg shadow-indigo-500/20">
            H
          </div>
          <div>
            <h1 className="font-semibold text-sm leading-none text-slate-100">
              Hydra Save Companion
            </h1>
            <span className="text-[10px] text-slate-400">Pop!_OS Edition</span>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {!loadingSettings && driveConnected && (
            <div className="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-slate-800/80 border border-slate-700/60 text-[11px] text-slate-300">
              <RefreshCw
                className={`w-3 h-3 text-indigo-400 ${!isManual ? "animate-spin-slow" : ""}`}
              />
              <span>
                {isManual
                  ? "Synchro manuelle"
                  : `Prochaine synchro : ${timeLeft}`}
              </span>
            </div>
          )}
          <button
            type="button"
            onClick={() => {
              if (driveConnected) {
                if (confirm("Veux-tu te déconnecter de Google Drive ?")) {
                  handleDisconnect();
                }
              } else {
                handleGoogleLogin();
              }
            }}
            disabled={loading}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-medium border transition-all cursor-pointer disabled:opacity-50 ${
              driveConnected
                ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/20"
                : "bg-rose-500/10 border-rose-500/20 text-rose-400 hover:bg-rose-500/20"
            }`}
            title={
              driveConnected
                ? "Connecté (Cliquer pour déconnecter)"
                : "Se connecter à Google Drive"
            }
          >
            {driveConnected ? (
              <Cloud className="w-3.5 h-3.5" />
            ) : (
              <CloudOff className="w-3.5 h-3.5" />
            )}
            <span>
              {loading
                ? "Connexion..."
                : driveConnected
                  ? "Drive actif"
                  : "Connecter Drive"}
            </span>
          </button>

          <button
            type="button"
            onClick={() => setIsSettingsOpen(true)}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition cursor-pointer"
            title="Paramètres"
          >
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </header>

      {isSettingsOpen && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md shadow-2xl overflow-hidden flex flex-col">
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800">
              <h2 className="text-base font-semibold text-slate-100">
                Paramètres
              </h2>
              <button
                type="button"
                onClick={() => setIsSettingsOpen(false)}
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

                  <div className="flex items-center justify-between pt-2">
                    <div>
                      <span className="text-sm font-medium text-slate-200 block">
                        Démarrage automatique
                      </span>
                      <span className="text-xs text-slate-400">
                        Lancer l'application avec le système
                      </span>
                    </div>
                    <input
                      type="checkbox"
                      checked={settings.autoStartWithSystem}
                      onChange={(e) =>
                        updateSettings({
                          autoStartWithSystem: e.target.checked,
                        })
                      }
                      className="w-4 h-4 accent-indigo-600 rounded cursor-pointer"
                    />
                  </div>
                </>
              )}
            </div>

            <div className="px-6 py-4 border-t border-slate-800 flex justify-end gap-3 bg-slate-950/50">
              <button
                type="button"
                onClick={() => setIsSettingsOpen(false)}
                className="px-4 py-2 rounded-xl text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-200 transition cursor-pointer"
              >
                Fermer
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
