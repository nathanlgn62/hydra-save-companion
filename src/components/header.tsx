import { Cloud, CloudOff, RefreshCw, Settings } from "lucide-react";
import { useState } from "react";
import { useAutoDownloadSaves } from "../hooks/useAutoSync";
import { useSettings } from "../hooks/useSettings";
import { useCloudStatus } from "../stores/cloudStore";
import SettingsModal from "./setting/setting-modal";

export default function Header() {
  const [loading] = useState<boolean>(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);

  const { timeLeft, isManual } = useAutoDownloadSaves();
  const { settings, updateSettings, loading: loadingSettings } = useSettings();
  const { isDriveConnected, loginGoogle, disconnectGoogle } = useCloudStatus();

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
          {!loadingSettings && isDriveConnected && (
            /* Le conteneur parent possède la classe "group" pour piloter le survol */
            <div className="group relative hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-slate-800/80 border border-slate-700/60 text-[11px] text-slate-300 cursor-default">
              <RefreshCw
                className={`w-3 h-3 text-indigo-400 ${!isManual ? "animate-spin-slow" : ""}`}
              />
              <span>
                {isManual
                  ? "Synchro manuelle"
                  : `Synchronisation depuis le cloud dans : ${timeLeft}`}
              </span>

              {/* Popup / Infobulle custom intégrée et positionnée proprement au-dessus */}
              <div className="absolute top-full left-1/2 -translate-x-1/2 mt-2 hidden group-hover:block w-60 p-2 bg-slate-950 text-slate-200 text-xs rounded-md shadow-xl border border-slate-800 text-center z-50 pointer-events-none">
                {isManual
                  ? "Les sauvegardes sont synchronisées manuellement."
                  : `Les sauvegardes du cloud seront automatiquement téléchargées toutes les ${settings.downloadInterval} minutes.`}
                <div className="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-slate-950"></div>
              </div>
            </div>
          )}

          <button
            type="button"
            onClick={() => {
              if (isDriveConnected) {
                if (confirm("Veux-tu te déconnecter de Google Drive ?")) {
                  disconnectGoogle();
                }
              } else {
                loginGoogle();
              }
            }}
            disabled={loading}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-medium border transition-all cursor-pointer disabled:opacity-50 ${
              isDriveConnected
                ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/20"
                : "bg-rose-500/10 border-rose-500/20 text-rose-400 hover:bg-rose-500/20"
            }`}
            title={
              isDriveConnected
                ? "Connecté (Cliquer pour déconnecter)"
                : "Se connecter à Google Drive"
            }
          >
            {isDriveConnected ? (
              <Cloud className="w-3.5 h-3.5" />
            ) : (
              <CloudOff className="w-3.5 h-3.5" />
            )}
            <span>
              {loading
                ? "Connexion..."
                : isDriveConnected
                  ? "Cloud connecté"
                  : "Connecter le cloud"}
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

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        settings={settings}
        updateSettings={updateSettings}
        loadingSettings={loadingSettings}
      />
    </>
  );
}
