import { invoke } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  Cloud,
  FolderOpen,
  Gamepad,
  HardDrive,
  Send,
  X,
} from "lucide-react";
import { useState } from "react";
import { HydraGame } from "../types/game";

interface GameCardProps {
  game: HydraGame;
}

export default function GameCard({ game }: GameCardProps) {
  const [isReportOpen, setIsReportOpen] = useState(false);
  const [message, setMessage] = useState("");
  const [isSending, setIsSending] = useState(false);
  const [sentSuccess, setSentSuccess] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);

  const handleOpenFolder = async () => {
    if (!game.savePath) return;
    try {
      await invoke("open_folder", { path: game.savePath });
    } catch (err) {
      console.error("Erreur ouverture dossier :", err);
    }
  };

  const handleSync = async () => {
    const storedToken = localStorage.getItem("gdrive_token");
    if (!storedToken) {
      alert("Veuillez vous connecter à Google Drive d'abord.");
      return;
    }

    const tokenParts = storedToken.split("|");
    const accessToken = tokenParts[0];

    setIsSyncing(true);
    try {
      // Pour l'instant, on envoie un texte de test basé sur le jeu
      const dummyContent = `Sauvegarde de test pour le jeu : ${game.title} (AppID: ${game.appId || "N/A"})`;

      const result = await invoke<string>("upload_game_save_to_drive", {
        token: accessToken,
        gameTitle: game.title,
        saveContent: dummyContent,
      });

      console.log(result);
      alert(`Synchronisation réussie pour ${game.title} !`);
    } catch (err) {
      console.error("Erreur lors de la synchronisation :", err);
      alert(`Échec de la synchronisation : ${err}`);
    } finally {
      setIsSyncing(false);
    }
  };

  const handleSendReport = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim()) return;

    setIsSending(true);

    try {
      await fetch("TON_WEBHOOK_DISCORD_OU_API_ICI", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          game: game.title,
          appId: game.appId,
          savePath: game.savePath,
          message: message,
        }),
      });

      setSentSuccess(true);
      setTimeout(() => {
        setIsReportOpen(false);
        setSentSuccess(false);
        setMessage("");
      }, 2000);
    } catch (err) {
      console.error("Erreur lors de l'envoi du report :", err);
    } finally {
      setIsSending(false);
    }
  };

  return (
    <>
      <div className="w-[calc(33.333%-11px)] bg-slate-900/90 border border-slate-800/80 rounded-xl overflow-hidden hover:border-indigo-500/50 transition-all duration-300 hover:shadow-xl hover:shadow-indigo-500/10 flex flex-col group">
        <div className="relative w-full aspect-[2/2] bg-gradient-to-b from-slate-900 to-slate-950 flex flex-col items-center justify-center p-4 text-center border-b border-slate-800/60 overflow-hidden">
          <div className="absolute inset-0 bg-[radial-gradient(#1e293b_1px,transparent_1px)] [background-size:12px_12px] opacity-40 pointer-events-none" />

          <div className="relative z-10 flex flex-col items-center gap-3 group-hover:scale-105 transition-transform duration-300">
            <div className="w-12 h-12 rounded-xl bg-slate-800/90 border border-slate-700/60 flex items-center justify-center text-slate-400 group-hover:border-indigo-500/50 group-hover:text-indigo-400 shadow-inner transition-colors">
              <Gamepad size={24} />
            </div>
            <span className="text-xs font-medium text-slate-300 line-clamp-2 leading-snug px-2">
              {game.title}
            </span>
          </div>
        </div>

        <div className="p-3 bg-slate-900 flex flex-col gap-2.5 flex-1 justify-between">
          <h3 className="font-semibold text-sm text-slate-100 truncate group-hover:text-indigo-300 transition-colors">
            {game.title}
          </h3>

          <div className="flex flex-col gap-1.5 text-[11px] text-slate-400 border-t border-slate-800/60 pt-2">
            <div className="flex items-center justify-between">
              <span className="flex items-center gap-1.5 text-slate-400">
                <HardDrive className="w-3.5 h-3.5 text-slate-500" />
                <span>Locale :</span>
              </span>
              <span className="text-slate-300 font-mono text-[10px]">
                {game.lastLocalSave ?? "Jamais"}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="flex items-center gap-1.5 text-slate-400">
                <Cloud className="w-3.5 h-3.5 text-slate-500" />
                <span>Distante :</span>
              </span>
              <span className="text-slate-300 font-mono text-[10px]">
                {game.lastRemoteSave ?? "Jamais"}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2 mt-1">
            <button
              type="button"
              onClick={handleOpenFolder}
              disabled={!game.savePath}
              title={
                game.savePath
                  ? `Ouvrir ${game.savePath}`
                  : "Aucun dossier trouvé"
              }
              className="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 border border-slate-700/60 text-slate-300 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed transition-all cursor-pointer flex items-center justify-center shrink-0"
            >
              <FolderOpen className="w-3.5 h-3.5" />
            </button>

            <button
              type="button"
              onClick={() => setIsReportOpen(true)}
              title="Signaler un problème"
              className="p-2 rounded-lg bg-slate-800 hover:bg-red-500/20 border border-slate-700/60 text-slate-400 hover:text-red-400 transition-all cursor-pointer flex items-center justify-center shrink-0"
            >
              <AlertTriangle className="w-3.5 h-3.5" />
            </button>

            <button
              type="button"
              onClick={handleSync}
              disabled={isSyncing}
              className="flex-1 py-1.5 px-3 rounded-lg bg-indigo-600 hover:bg-indigo-500 active:scale-[0.98] disabled:opacity-50 text-white font-medium text-xs flex items-center justify-center gap-2 shadow-sm shadow-indigo-900/30 transition-all cursor-pointer"
            >
              {isSyncing ? "Sync..." : "Synchroniser"}
            </button>
          </div>
        </div>
      </div>

      {isReportOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-fadeIn">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md overflow-hidden shadow-2xl flex flex-col">
            <div className="flex items-center justify-between px-5 py-4 border-b border-slate-800">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400">
                  <AlertTriangle className="w-4 h-4" />
                </div>
                <div>
                  <h3 className="font-semibold text-sm text-slate-100">
                    Signaler un problème
                  </h3>
                  <p className="text-xs text-slate-400">{game.title}</p>
                </div>
              </div>
              <button
                type="button"
                onClick={() => setIsReportOpen(false)}
                className="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {sentSuccess ? (
              <div className="p-8 text-center flex flex-col items-center gap-3">
                <div className="w-12 h-12 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center text-lg font-bold">
                  ✓
                </div>
                <p className="text-sm font-medium text-slate-200">
                  Message envoyé avec succès !
                </p>
                <p className="text-xs text-slate-400">
                  Merci pour ton retour, le problème a bien été transmis.
                </p>
              </div>
            ) : (
              <form
                onSubmit={handleSendReport}
                className="p-5 flex flex-col gap-4"
              >
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-medium text-slate-300">
                    Décris le problème rencontré (ex: chemin de sauvegarde
                    introuvable) :
                  </label>
                  <textarea
                    rows={4}
                    value={message}
                    onChange={(e) => setMessage(e.target.value)}
                    placeholder="Explique ce qui ne va pas..."
                    className="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500 transition-colors resize-none"
                    required
                  />
                </div>

                <div className="flex items-center justify-end gap-2.5 pt-2 border-t border-slate-800">
                  <button
                    type="button"
                    onClick={() => setIsReportOpen(false)}
                    className="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
                  >
                    Annuler
                  </button>
                  <button
                    type="submit"
                    disabled={isSending || !message.trim()}
                    className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed text-white text-xs font-medium flex items-center gap-2 shadow-sm shadow-indigo-900/30 transition-all"
                  >
                    {isSending ? (
                      <>Envoi en cours...</>
                    ) : (
                      <>
                        <Send className="w-3.5 h-3.5" />
                        Envoyer le rapport
                      </>
                    )}
                  </button>
                </div>
              </form>
            )}
          </div>
        </div>
      )}
    </>
  );
}
