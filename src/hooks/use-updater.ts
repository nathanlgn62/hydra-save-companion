import { useCallback, useEffect, useRef, useState } from "react";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { exit, relaunch } from "@tauri-apps/plugin-process";

export type UpdaterStatus =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "ready"
  | "error";

export interface DownloadProgress {
  downloadedBytes: number;
  totalBytes: number;
  percent: number;
}

export function useUpdater() {
  const [status, setStatus] = useState<UpdaterStatus>("idle");
  const [updateInfo, setUpdateInfo] = useState<Update | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<DownloadProgress>({
    downloadedBytes: 0,
    totalBytes: 0,
    percent: 0,
  });

  const activeUpdateRef = useRef<Update | null>(null);

  const checkForUpdates = useCallback(async () => {
    try {
      setStatus("checking");
      setError(null);

      const update = await check();

      if (update) {
        activeUpdateRef.current = update;
        setUpdateInfo(update);
        setStatus("available");
        return update;
      }

      activeUpdateRef.current = null;
      setUpdateInfo(null);
      setStatus("idle");
      return null;
    } catch (err: unknown) {
      const message =
        err instanceof Error
          ? err.message
          : typeof err === "string"
            ? err
            : "Failed to check for updates";

      // In dev mode or offline without releases yet, log a warning instead of blocking
      console.warn("[Updater] Check failed:", message);
      setError(message);
      setStatus("idle");
      return null;
    }
  }, []);

  const startDownloadAndInstall = useCallback(async () => {
    const update = activeUpdateRef.current;
    if (!update) return;

    try {
      setStatus("downloading");
      setError(null);
      setProgress({ downloadedBytes: 0, totalBytes: 0, percent: 0 });

      let downloaded = 0;
      let total = 0;

      await update.download((event: DownloadEvent) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? 0;
          setProgress({
            downloadedBytes: 0,
            totalBytes: total,
            percent: 0,
          });
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          const percent = total > 0 ? Math.min(Math.round((downloaded / total) * 100), 100) : 0;
          setProgress({
            downloadedBytes: downloaded,
            totalBytes: total,
            percent,
          });
        } else if (event.event === "Finished") {
          setProgress((prev) => ({ ...prev, percent: 100 }));
        }
      });

      setStatus("installing");
      await update.install();

      setStatus("ready");

      // Give users a brief moment to see completion, then restart
      setTimeout(async () => {
        try {
          await relaunch();
        } catch (relaunchErr) {
          console.warn("[Updater] Relaunch initiated or already handled:", relaunchErr);
        }
      }, 1200);
    } catch (err: unknown) {
      const message =
        err instanceof Error
          ? err.message
          : typeof err === "string"
            ? err
            : "Failed to download and install update";

      console.error("[Updater] Update failed:", message);
      setError(message);
      setStatus("error");
    }
  }, []);

  const quitApp = useCallback(async () => {
    try {
      await exit(0);
    } catch {
      window.close();
    }
  }, []);

  const restartApp = useCallback(async () => {
    try {
      await relaunch();
    } catch (err) {
      console.error("[Updater] Relaunch failed:", err);
    }
  }, []);

  // Cleanup update resource if unmounted before installing
  useEffect(() => {
    return () => {
      if (activeUpdateRef.current) {
        activeUpdateRef.current.close().catch(() => {});
      }
    };
  }, []);

  return {
    status,
    updateInfo,
    error,
    progress,
    checkForUpdates,
    startDownloadAndInstall,
    quitApp,
    restartApp,
  };
}

