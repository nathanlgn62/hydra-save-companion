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
} from "lucide-react";
import { useState } from "react";
import {
  useDownloadSave,
  useGameSyncStatus,
  useUploadSave,
} from "../hooks/useGames";
import { useIsGameDownloading } from "../stores/gameStore";
import { HydraGame } from "../types/game";
import { formatDate } from "../utils/date";
import { openFolder } from "../utils/openFolder";
import { getSyncBadgeConfig } from "../utils/syncBadge";
import GameReportModal from "./game/game-report-modal";
import SyncBadge from "./sync-badge";

interface GameCardProps {
  game: HydraGame;
}

export default function GameCard({ game }: GameCardProps) {
  const [isReportOpen, setIsReportOpen] = useState(false);

  const syncMutation = useUploadSave();
  const downloadMutation = useDownloadSave();
  const syncDirection = useGameSyncStatus(
    game.lastLocalSave,
    game.lastRemoteSave,
  );
  const isAutoDownloading = useIsGameDownloading(game.title);

  const badgeConfig = getSyncBadgeConfig(
    game.ludasaviPathExists,
    game.lastLocalSave,
    game.lastRemoteSave,
  );

  const countdown = null;
  const clearCountdown = () => {};

  const handleSync = () => {
    if (
      !game.savePath ||
      syncMutation.isPending ||
      downloadMutation.isPending
    ) {
      return;
    }

    clearCountdown();

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
          {game.cover && (
            <>
              <div
                className="absolute inset-0 bg-cover bg-center opacity-30 group-hover:opacity-40 group-hover:scale-105 transition-all duration-500"
                style={{ backgroundImage: `url(${game.cover})` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-slate-950 via-slate-950/60 to-slate-950/30" />
            </>
          )}

          <div className="absolute top-3 right-3 z-20">
            <SyncBadge config={badgeConfig} />
          </div>

          {!game.cover && (
            <div className="absolute inset-0 bg-[radial-gradient(#1e293b_1px,transparent_1px)] [background-size:12px_12px] opacity-40 pointer-events-none" />
          )}

          <div className="relative z-10 flex flex-col items-center gap-3 group-hover:scale-105 transition-transform duration-300">
            <div className="w-12 h-12 rounded-xl bg-slate-800/90 border border-slate-700/60 flex items-center justify-center text-slate-400 group-hover:border-indigo-500/50 group-hover:text-indigo-400 shadow-inner transition-colors">
              <Gamepad size={24} />
            </div>
            <span className="text-xs font-medium text-slate-300 line-clamp-2 leading-snug px-2 drop-shadow-md">
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

      <GameReportModal
        gameTitle={game.title}
        gameObjectId={game.objectId}
        savePath={game.savePath}
        isOpen={isReportOpen}
        onClose={() => setIsReportOpen(false)}
      />
    </>
  );
}
