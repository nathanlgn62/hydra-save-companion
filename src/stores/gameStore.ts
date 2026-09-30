import { useEffect, useState } from "react";

type SyncStatusListener = (activeGames: Record<string, boolean>) => void;

let activeDownloads: Record<string, boolean> = {};
const listeners = new Set<SyncStatusListener>();

export const setGameDownloading = (
  gameTitle: string,
  isDownloading: boolean,
) => {
  if (isDownloading) {
    activeDownloads = { ...activeDownloads, [gameTitle]: true };
  } else {
    const copy = { ...activeDownloads };
    delete copy[gameTitle];
    activeDownloads = copy;
  }
  listeners.forEach((listener) => listener(activeDownloads));
};

export const subscribeToSyncStatus = (listener: SyncStatusListener) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

export const getActiveDownloads = () => activeDownloads;

export function useIsGameDownloading(gameTitle: string) {
  const [isDownloading, setIsDownloading] = useState(
    !!getActiveDownloads()[gameTitle],
  );

  useEffect(() => {
    return subscribeToSyncStatus((downloads) => {
      setIsDownloading(!!downloads[gameTitle]);
    });
  }, [gameTitle]);

  return isDownloading;
}
