import {
  AlertTriangle,
  Clock,
  Cloud,
  CloudDownload,
  CloudUpload,
  FolderOpen,
  Gamepad,
  HardDrive,
  History,
  Loader2,
  RefreshCw,
} from "lucide-react";
import { useEffect, useState } from "react";
import {
  useDownloadSave,
  useGameSyncStatus,
  useUploadSave,
} from "../hooks/use-games";
import { useIsGameDownloading } from "../stores/game-store";
import { HydraGame } from "../types";
import { formatDate } from "../utils/date";
import { openFolder } from "../utils/open-folder";
import { getSyncBadgeConfig } from "../utils/sync-badge";
import GameBackupsModal from "./game/game-backups-modal";
import GameReportModal from "./game/game-report-modal";
import SyncBadge from "./sync-badge";

interface GameCardProps {
  game: HydraGame;
}

export default function GameCard({ game }: GameCardProps) {
  const [isReportOpen, setIsReportOpen] = useState(false);
  const [isBackupsOpen, setIsBackupsOpen] = useState(false);
  const [imgError, setImgError] = useState(false);

  useEffect(() => {
    setImgError(false);
  }, [game.cover]);

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

  const hasBackups = Boolean(game.backups && game.backups.length > 0);
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
        onError: (err: unknown) => {
          const msg = err instanceof Error ? err.message : String(err);
          console.error(msg);
        },
      });
    } else {
      syncMutation.mutate(payload, {
        onError: (err: unknown) => {
          const msg = err instanceof Error ? err.message : String(err);
          console.error(msg);
        },
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
      <div className="w-full bg-slate-900/95 border border-slate-800/90 rounded-2xl overflow-hidden hover:border-indigo-500/50 transition-all duration-300 hover:shadow-2xl hover:shadow-indigo-500/10 flex flex-col group">
        {/* Zone jaquette au ratio 2:3 (format officiel bibliothèque Steam & Hydra Launcher) */}
        <div className="relative w-full aspect-[2/3] bg-slate-950 flex items-center justify-center overflow-hidden">
          {game.cover && !imgError ? (
            <>
              {/* Jaquette officielle remplissant l'espace (object-cover) avec zoom subtil au survol */}
              <img
                src={game.cover}
                alt={game.title}
                onError={() => setImgError(true)}
                className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500 ease-out"
                loading="lazy"
              />
              {/* Dégradé supérieur pour détacher le badge */}
              <div className="absolute inset-x-0 top-0 h-16 bg-gradient-to-b from-black/80 via-black/30 to-transparent pointer-events-none z-10" />
              {/* Dégradé inférieur pour une transition douce vers les actions */}
              <div className="absolute inset-x-0 bottom-0 h-16 bg-gradient-to-t from-slate-900 via-slate-900/40 to-transparent pointer-events-none z-10" />
            </>
          ) : (
            <div className="relative z-10 flex flex-col items-center gap-3 p-4 text-center">
              <div className="w-14 h-14 rounded-2xl bg-slate-800/90 border border-slate-700/60 flex items-center justify-center text-slate-400 group-hover:border-indigo-500/50 group-hover:text-indigo-400 shadow-inner transition-colors">
                <Gamepad size={28} />
              </div>
              <span className="text-xs font-medium text-slate-300 line-clamp-2 leading-snug px-2 drop-shadow-md">
                {game.title}
              </span>
            </div>
          )}

          {/* Badge d'état de synchronisation en haut à droite */}
          <div className="absolute top-2.5 right-2.5 z-20">
            <SyncBadge config={badgeConfig} />
          </div>
        </div>

        <div className="p-3 bg-slate-900 flex flex-col gap-2.5 flex-1 justify-between">
          <div>
            <h3
              className="font-semibold text-sm text-slate-100 truncate group-hover:text-indigo-300 transition-colors"
              title={game.title}
            >
              {game.title}
            </h3>

            <div className="flex flex-col gap-1.5 text-[11px] text-slate-400 border-t border-slate-800/60 pt-2 mt-2">
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
          </div>

          <div className="flex flex-col gap-1.5 pt-1">
            <div className="grid grid-cols-3 gap-1.5 w-full">
              <button
                type="button"
                onClick={() => openFolder(game.savePath)}
                disabled={!game.localPathExists}
                title={
                  game.savePath
                    ? `Ouvrir ${game.savePath}`
                    : "Aucun dossier trouvé"
                }
                className="h-8 rounded-lg bg-slate-800 hover:bg-slate-700 border border-slate-700/60 text-slate-300 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed transition-all cursor-pointer flex items-center justify-center shrink-0"
              >
                <FolderOpen className="w-3.5 h-3.5" />
              </button>

              <button
                type="button"
                onClick={() => setIsBackupsOpen(true)}
                disabled={!hasBackups}
                title={
                  hasBackups
                    ? `Voir les sauvegardes cloud (${game.backups!.length})`
                    : "Aucune sauvegarde cloud disponible"
                }
                className="h-8 rounded-lg bg-slate-800 hover:bg-slate-700 border border-slate-700/60 text-slate-300 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed transition-all cursor-pointer flex items-center justify-center shrink-0"
              >
                <History className="w-3.5 h-3.5" />
              </button>

              <button
                type="button"
                onClick={() => setIsReportOpen(true)}
                title="Signaler un problème"
                className="h-8 rounded-lg bg-red-500/10 hover:bg-red-500/20 border border-red-500/30 hover:border-red-500/50 text-red-400 hover:text-red-300 transition-all cursor-pointer flex items-center justify-center shrink-0 shadow-sm shadow-red-950/20"
              >
                <AlertTriangle className="w-3.5 h-3.5" />
              </button>
            </div>

            <button
              type="button"
              onClick={handleSync}
              disabled={isSyncDisabled}
              className="w-full h-8 px-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100 text-white font-medium text-xs flex items-center justify-center gap-1.5 shadow-sm shadow-indigo-900/30 transition-all cursor-pointer"
            >
              {isPending ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin shrink-0" />
                  <span className="truncate">
                    {downloadMutation.isPending
                      ? "Téléchargement..."
                      : syncMutation.isPending
                      ? "Envoi vers le cloud..."
                      : "Synchronisation..."}
                  </span>
                </>
              ) : countdown !== null ? (
                <>
                  <Clock className="w-3.5 h-3.5 text-amber-300 animate-pulse shrink-0" />
                  <span className="truncate">Sync {countdown}s</span>
                </>
              ) : syncDirection === "down" ? (
                <>
                  <CloudDownload className="w-3.5 h-3.5 shrink-0" />
                  <span className="truncate">Synchroniser vers local</span>
                </>
              ) : syncDirection === "up" ? (
                <>
                  <CloudUpload className="w-3.5 h-3.5 shrink-0" />
                  <span className="truncate">Synchroniser vers cloud</span>
                </>
              ) : (
                <>
                  <RefreshCw className="w-3.5 h-3.5 shrink-0" />
                  <span className="truncate">Synchroniser</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      <GameBackupsModal
        gameTitle={game.title}
        savePath={game.savePath}
        lastLocalSave={game.lastLocalSave}
        backups={game.backups ?? []}
        isOpen={isBackupsOpen}
        onClose={() => setIsBackupsOpen(false)}
      />

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
