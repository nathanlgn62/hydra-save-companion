import type { Update } from "@tauri-apps/plugin-updater";
import {
  AlertCircle,
  ArrowRight,
  CheckCircle2,
  Download,
  RefreshCw,
  XCircle,
} from "lucide-react";
import type { DownloadProgress, UpdaterStatus } from "../../hooks/use-updater";

interface UpdateOverlayProps {
  status: UpdaterStatus;
  updateInfo: Update | null;
  progress: DownloadProgress;
  error: string | null;
  onDownloadAndInstall: () => void;
  onQuit: () => void;
  onRestart: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 MB";
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(1)} MB`;
}

export default function UpdateOverlay({
  status,
  updateInfo,
  progress,
  error,
  onDownloadAndInstall,
  onQuit,
  onRestart,
}: UpdateOverlayProps) {
  // Only render when an update is active or encountered an error
  if (status === "idle" || status === "checking" || !updateInfo) {
    return null;
  }

  const isDownloading = status === "downloading";
  const isInstalling = status === "installing";
  const isReady = status === "ready";
  const isError = status === "error";

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="update-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-[#090514]/95 backdrop-blur-md p-4 select-none animate-in fade-in duration-300"
    >
      {/* Background glow and ambient grid */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_35%,rgba(37,215,207,0.12),rgba(19,255,182,0.06)_40%,transparent_75%)] pointer-events-none" />
      <div className="absolute inset-0 bg-[linear-gradient(to_right,rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(to_bottom,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:3rem_3rem] pointer-events-none" />

      {/* Main modal card */}
      <div className="relative z-10 w-full max-w-lg bg-[#140827] border border-cyan-500/30 rounded-2xl p-6 shadow-2xl shadow-cyan-950/80 flex flex-col overflow-hidden">
        {/* Header with pill and title */}
        <div className="flex items-center justify-between mb-4">
          <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-cyan-950/70 border border-cyan-400/40 text-[11px] font-mono tracking-widest text-cyan-300 uppercase">
            <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-ping" />
            Mandatory Update
          </div>
          <span className="text-xs font-mono text-slate-400">
            Hydra Save Companion
          </span>
        </div>

        <h2
          id="update-title"
          className="text-xl font-bold tracking-tight text-white mb-2"
        >
          New version available
        </h2>
        <p className="text-xs text-slate-300 mb-5 leading-relaxed">
          A critical update is required to ensure save integrity and cloud
          synchronization compatibility. Please update before continuing.
        </p>

        {/* Version progression comparison card */}
        <div className="flex items-center justify-between px-4 py-3 bg-[#0c051a] border border-slate-800 rounded-xl mb-4 font-mono text-xs">
          <div className="flex flex-col">
            <span className="text-[10px] uppercase text-slate-400 tracking-wider">
              Installed
            </span>
            <span className="text-slate-300 font-semibold mt-0.5">
              v{updateInfo.currentVersion}
            </span>
          </div>

          <div className="p-1.5 rounded-full bg-cyan-950/50 border border-cyan-500/30 text-cyan-400">
            <ArrowRight className="w-4 h-4" />
          </div>

          <div className="flex flex-col text-right">
            <span className="text-[10px] uppercase text-emerald-400 tracking-wider">
              Latest Release
            </span>
            <span className="text-emerald-300 font-bold mt-0.5">
              v{updateInfo.version}
            </span>
          </div>
        </div>

        {/* Release notes preview if available */}
        {updateInfo.body ? (
          <div className="mb-5 flex flex-col">
            <span className="text-[11px] font-mono uppercase tracking-wider text-slate-400 mb-1.5">
              Changelog & Highlights
            </span>
            <div className="max-h-36 overflow-y-auto rounded-lg bg-[#0c051a]/70 border border-slate-800/80 p-3 text-xs text-slate-300 leading-relaxed font-sans whitespace-pre-wrap select-text scrollbar-thin">
              {updateInfo.body}
            </div>
          </div>
        ) : null}

        {/* Progress or Status Information */}
        {isDownloading || isInstalling || isReady ? (
          <div className="flex flex-col mb-5">
            <div className="flex items-center justify-between text-xs font-mono text-slate-300 mb-2">
              <span className="flex items-center gap-2 text-cyan-300 font-medium">
                {isDownloading && (
                  <>
                    <RefreshCw className="w-3.5 h-3.5 animate-spin text-cyan-400" />
                    Downloading update...
                  </>
                )}
                {isInstalling && (
                  <>
                    <RefreshCw className="w-3.5 h-3.5 animate-spin text-emerald-400" />
                    Installing & verifying files...
                  </>
                )}
                {isReady && (
                  <>
                    <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                    Update installed successfully!
                  </>
                )}
              </span>

              {isDownloading && progress.totalBytes > 0 && (
                <span className="tabular-nums text-slate-400">
                  {formatBytes(progress.downloadedBytes)} /{" "}
                  {formatBytes(progress.totalBytes)} ({progress.percent}%)
                </span>
              )}
            </div>

            {/* Progress bar */}
            <div className="w-full h-2 bg-slate-900 rounded-full overflow-hidden border border-slate-800 p-0.5 relative shadow-inner">
              <div
                className="h-full bg-gradient-to-r from-[#13FFB6] via-[#25D7CF] to-[#2FACF6] rounded-full transition-all duration-300 ease-out relative shadow-[0_0_10px_rgba(37,215,207,0.8)]"
                style={{
                  width: `${isInstalling || isReady ? 100 : progress.percent}%`,
                }}
              />
            </div>
          </div>
        ) : null}

        {/* Error message card */}
        {isError && (
          <div className="mb-5 flex items-start gap-2.5 p-3 rounded-xl bg-rose-950/40 border border-rose-500/40 text-rose-300 text-xs">
            <AlertCircle className="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
            <div className="flex flex-col">
              <span className="font-semibold text-rose-200">Update failed</span>
              <span className="text-[11px] text-rose-300/90 mt-0.5 break-words">
                {error ||
                  "An unknown error occurred while downloading the update."}
              </span>
            </div>
          </div>
        )}

        {/* Action buttons */}
        <div className="flex items-center justify-end gap-3 pt-2 border-t border-slate-800/80">
          <button
            type="button"
            onClick={onQuit}
            className="px-3.5 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-900/60 transition cursor-pointer flex items-center gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-600"
          >
            <XCircle className="w-4 h-4" />
            Quit App
          </button>

          {isReady ? (
            <button
              type="button"
              onClick={onRestart}
              className="px-4 py-2 rounded-xl text-xs font-semibold bg-emerald-500 hover:bg-emerald-400 text-slate-950 shadow-lg shadow-emerald-500/20 transition cursor-pointer flex items-center gap-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400"
            >
              <RefreshCw className="w-4 h-4" />
              Restart Now
            </button>
          ) : (
            <button
              type="button"
              onClick={onDownloadAndInstall}
              disabled={isDownloading || isInstalling}
              className="px-4 py-2 rounded-xl text-xs font-semibold bg-cyan-500 hover:bg-cyan-400 disabled:opacity-50 disabled:cursor-not-allowed text-slate-950 shadow-lg shadow-cyan-500/25 transition cursor-pointer flex items-center gap-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
            >
              {isDownloading || isInstalling ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Updating...
                </>
              ) : isError ? (
                <>
                  <RefreshCw className="w-4 h-4" />
                  Retry Update
                </>
              ) : (
                <>
                  <Download className="w-4 h-4" />
                  Download & Install
                </>
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
