import { Cloud, Download, History, Loader2, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useDownloadSave } from "../../hooks/useGames";
import { RemoteBackupInfo } from "../../types/game";

interface GameBackupsModalProps {
  gameTitle: string;
  savePath: string | null;
  backups: RemoteBackupInfo[];
  isOpen: boolean;
  onClose: () => void;
}

export default function GameBackupsModal({
  gameTitle,
  savePath,
  backups,
  isOpen,
  onClose,
}: GameBackupsModalProps) {
  const downloadMutation = useDownloadSave();
  const [restoringFileId, setRestoringFileId] = useState<string | null>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const handleRestore = (backup: RemoteBackupInfo) => {
    if (!savePath || downloadMutation.isPending) return;

    setRestoringFileId(backup.file_id);
    downloadMutation.mutate(
      {
        gameTitle,
        savePath,
        fileId: backup.file_id,
      },
      {
        onSettled: () => {
          setRestoringFileId(null);
        },
      },
    );
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-200">
      <div
        className="w-full max-w-lg bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-200"
        role="dialog"
        aria-modal="true"
      >
        {/* Header */}
        <div className="p-4 border-b border-slate-800 flex items-center justify-between bg-slate-900/50">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-lg bg-indigo-500/10 border border-indigo-500/20 text-indigo-400">
              <History className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-slate-100 flex items-center gap-2">
                Sauvegardes Cloud
              </h2>
              <p className="text-xs text-slate-400 truncate max-w-xs sm:max-w-md">
                {gameTitle}
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 overflow-y-auto space-y-2.5 flex-1 custom-scrollbar">
          {backups.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-10 text-center gap-2">
              <Cloud className="w-10 h-10 text-slate-600 mb-1" />
              <p className="text-sm font-medium text-slate-300">
                Aucune sauvegarde sur le cloud
              </p>
              <p className="text-xs text-slate-500 max-w-xs">
                Synchronisez vos sauvegardes locales pour qu'elles apparaissent ici.
              </p>
            </div>
          ) : (
            backups.map((backup, index) => {
              const isRestoring =
                downloadMutation.isPending && restoringFileId === backup.file_id;

              return (
                <div
                  key={backup.file_id || index}
                  className="flex items-center justify-between p-3 rounded-xl bg-slate-800/60 border border-slate-700/50 hover:border-slate-600/60 transition-all gap-3"
                >
                  <div className="flex flex-col min-w-0">
                    <span className="text-xs font-medium text-slate-200 truncate">
                      {backup.name}
                    </span>
                    <span className="text-[11px] font-mono text-slate-400 mt-0.5">
                      {backup.modified_time}
                    </span>
                  </div>

                  <button
                    type="button"
                    onClick={() => handleRestore(backup)}
                    disabled={!savePath || downloadMutation.isPending}
                    title={
                      !savePath
                        ? "Dossier local de sauvegarde introuvable"
                        : "Restaurer cette version"
                    }
                    className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100 text-white font-medium text-xs flex items-center gap-1.5 transition-all shrink-0 cursor-pointer shadow-sm shadow-indigo-900/30"
                  >
                    {isRestoring ? (
                      <>
                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        <span>Restauration...</span>
                      </>
                    ) : (
                      <>
                        <Download className="w-3.5 h-3.5" />
                        <span>Restaurer</span>
                      </>
                    )}
                  </button>
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="p-3 bg-slate-950/40 border-t border-slate-800 flex justify-between items-center text-xs text-slate-500">
          <span>{backups.length} sauvegarde{backups.length > 1 ? "s" : ""} trouvée{backups.length > 1 ? "s" : ""}</span>
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors cursor-pointer text-xs"
          >
            Fermer
          </button>
        </div>
      </div>
    </div>
  );
}

