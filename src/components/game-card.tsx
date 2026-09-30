import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import {
  AlertTriangle,
  Clock,
  Cloud,
  Download,
  FolderOpen,
  Gamepad,
  HardDrive,
  Loader2,
  RefreshCw,
  Send,
  X,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useDownloadGame, useSyncGame } from "../hooks/useGames";
import { useSettings } from "../hooks/useSettings";
import { useIsGameDownloading } from "../stores/gameStore";
import { HydraGame } from "../types/game";
import { openFolder } from "../utils/openFolder";
import { getSyncBadgeConfig } from "../utils/syncBadge";
import SyncBadge from "./sync-badge";

interface GameCardProps {
  game: HydraGame;
}

interface GameClosedPayload {
  title: string;
  save_path: string | null;
}

const parseSaveDate = (dateStr?: string | null): number | null => {
  if (!dateStr || dateStr === "Jamais" || dateStr === "Aucune") return null;

  // Format Rust "DD/MM/YYYY HH:mm"
  const customFormatRegex = /^(\d{2})\/(\d{2})\/(\d{4})\s+(\d{2}):(\d{2})$/;
  const match = dateStr.match(customFormatRegex);

  if (match) {
    const [, day, month, year, hours, minutes] = match;
    return new Date(
      Number(year),
      Number(month) - 1,
      Number(day),
      Number(hours),
      Number(minutes),
    ).getTime();
  }

  // Fallback ISO
  const parsed = new Date(dateStr).getTime();
  return isNaN(parsed) ? null : parsed;
};

const formatDate = (dateStr?: string | null) => {
  if (!dateStr) return "Jamais";
  try {
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    return new Intl.DateTimeFormat("fr-FR", {
      day: "2-digit",
      month: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    }).format(date);
  } catch {
    return dateStr;
  }
};

export default function GameCard({ game }: GameCardProps) {
  const [isReportOpen, setIsReportOpen] = useState(false);
  const [message, setMessage] = useState("");
  const [isSending, setIsSending] = useState(false);
  const [sentSuccess, setSentSuccess] = useState(false);

  const [countdown, setCountdown] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const syncMutation = useSyncGame();
  const downloadMutation = useDownloadGame();
  const queryClient = useQueryClient();
  const { settings } = useSettings();

  const syncMutationRef = useRef(syncMutation);

  const isAutoDownloading = useIsGameDownloading(game.title);

  const badgeConfig = getSyncBadgeConfig(
    game.ludasaviPathExists,
    game.lastLocalSave,
    game.lastRemoteSave,
  );

  // Détermination du sens de synchronisation
  const syncDirection = useMemo(() => {
    const localTime = parseSaveDate(game.lastLocalSave);
    const remoteTime = parseSaveDate(game.lastRemoteSave);

    if (!remoteTime && !localTime) return "none";
    if (!localTime && remoteTime) return "down";
    if (localTime && !remoteTime) return "up";

    if (remoteTime! > localTime!) return "down";
    if (localTime! > remoteTime!) return "up";
    return "synced";
  }, [game.lastLocalSave, game.lastRemoteSave]);

  useEffect(() => {
    let unlistenStartedFn: (() => void) | null = null;
    let unlistenClosedFn: (() => void) | null = null;

    const setupListeners = async () => {
      unlistenStartedFn = await listen<string>("game-started", (event) => {
        if (event.payload === game.title) {
          if (timerRef.current) {
            clearInterval(timerRef.current);
            timerRef.current = null;
          }
          setCountdown(null);
        }
      });

      unlistenClosedFn = await listen<GameClosedPayload>(
        "game-closed",
        (event) => {
          if (event.payload.title !== game.title || !game.savePath) {
            return;
          }

          if (settings.uploadInterval === "afterGameClose") {
            if (timerRef.current) {
              clearInterval(timerRef.current);
            }

            setCountdown(45);

            const intervalId = setInterval(() => {
              setCountdown((prev) => {
                if (prev === null || prev <= 1) {
                  clearInterval(intervalId);
                  timerRef.current = null;

                  syncMutationRef.current.mutate({
                    gameTitle: game.title,
                    savePath: game.savePath!,
                  });

                  return null;
                }
                return prev - 1;
              });
            }, 1000);

            timerRef.current = intervalId;
          } else {
            queryClient.invalidateQueries({ queryKey: ["games"] });
          }
        },
      );
    };

    setupListeners();

    return () => {
      if (unlistenStartedFn) unlistenStartedFn();
      if (unlistenClosedFn) unlistenClosedFn();
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [game.title, game.savePath, settings.uploadInterval, queryClient]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isReportOpen) {
        handleCloseReport();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isReportOpen]);

  useEffect(() => {
    syncMutationRef.current = syncMutation;
  }, [syncMutation]);

  const handleCloseReport = () => {
    setIsReportOpen(false);
    setSentSuccess(false);
    setMessage("");
  };

  const handleSync = () => {
    if (
      !game.savePath ||
      syncMutation.isPending ||
      downloadMutation.isPending
    ) {
      return;
    }

    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
      setCountdown(null);
    }

    const payload = { gameTitle: game.title, savePath: game.savePath };

    if (syncDirection === "down") {
      downloadMutation.mutate(payload, {
        onError: (err: any) =>
          console.log(
            typeof err === "string" ? err : err?.message || String(err),
          ),
      });
    } else {
      syncMutation.mutate(payload, {
        onError: (err: any) =>
          console.log(
            typeof err === "string" ? err : err?.message || String(err),
          ),
      });
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
          appId: game.objectId,
          savePath: game.savePath,
          message: message.trim(),
        }),
      });

      setSentSuccess(true);
      setTimeout(() => {
        handleCloseReport();
      }, 2000);
    } catch (err) {
      console.error("Erreur lors de l'envoi du report :", err);
    } finally {
      setIsSending(false);
    }
  };

  const isPending =
    syncMutation.isPending || downloadMutation.isPending || isAutoDownloading;

  const isSyncDisabled =
    isPending ||
    !game.savePath ||
    (countdown === null &&
      (syncDirection === "synced" || syncDirection === "none"));

  return (
    <>
      <div className="w-[calc(33.333%-11px)] bg-slate-900/90 border border-slate-800/80 rounded-xl overflow-hidden hover:border-indigo-500/50 transition-all duration-300 hover:shadow-xl hover:shadow-indigo-500/10 flex flex-col group">
        <div className="relative w-full aspect-[2/2] bg-gradient-to-b from-slate-900 to-slate-950 flex flex-col items-center justify-center p-4 text-center border-b border-slate-800/60 overflow-hidden">
          <div className="absolute top-3 right-3 z-20">
            <SyncBadge config={badgeConfig} />
          </div>

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
                {formatDate(game.lastLocalSave)}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="flex items-center gap-1.5 text-slate-400">
                <Cloud className="w-3.5 h-3.5 text-slate-500" />
                <span>Distante :</span>
              </span>
              <span className="text-slate-300 font-mono text-[10px]">
                {formatDate(game.lastRemoteSave)}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2 mt-1">
            <button
              type="button"
              onClick={() => openFolder(game.savePath)}
              disabled={!game.localPathExists}
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
              disabled={isSyncDisabled}
              className="flex-1 py-1.5 px-3 rounded-lg bg-indigo-600 hover:bg-indigo-500 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100 text-white font-medium text-xs flex items-center justify-center gap-2 shadow-sm shadow-indigo-900/30 transition-all cursor-pointer"
            >
              {isPending ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  <span>
                    {downloadMutation.isPending
                      ? "Téléchargement..."
                      : "Sync..."}
                  </span>
                </>
              ) : countdown !== null ? (
                <>
                  <Clock className="w-3.5 h-3.5 text-amber-300 animate-pulse" />
                  <span>Sync dans {countdown}s</span>
                </>
              ) : syncDirection === "down" ? (
                <>
                  <Download className="w-3.5 h-3.5" />
                  <span>Télécharger</span>
                </>
              ) : (
                <>
                  <RefreshCw className="w-3.5 h-3.5" />
                  <span>Synchroniser</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Modal Report */}
      {isReportOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-fadeIn"
          onClick={handleCloseReport}
        >
          <div
            className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md overflow-hidden shadow-2xl flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
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
                onClick={handleCloseReport}
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
                    onClick={handleCloseReport}
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
