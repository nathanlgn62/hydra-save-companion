import { invoke } from "@tauri-apps/api/core";
import { Cloud, CloudOff, Settings } from "lucide-react";
import { useEffect, useState } from "react";

export default function Header() {
  const [driveConnected, setDriveConnected] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);

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

  return (
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
          onClick={() => alert("Paramètres généraux à venir")}
          className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition cursor-pointer"
          title="Paramètres"
        >
          <Settings className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
}
