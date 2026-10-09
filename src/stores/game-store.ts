import { useEffect, useState } from "react";

type SyncStatusListener = (activeGames: Record<string, boolean>) => void;
type RunningGameListener = (gameName: string | null) => void;

let activeDownloads: Record<string, boolean> = {};
let runningGame: string | null = null;

const syncListeners = new Set<SyncStatusListener>();
const runningListeners = new Set<RunningGameListener>();

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
  syncListeners.forEach((listener) => listener(activeDownloads));
};

export const subscribeToSyncStatus = (listener: SyncStatusListener) => {
  syncListeners.add(listener);
  return () => {
    syncListeners.delete(listener);
  };
};

export const getActiveDownloads = () => activeDownloads;

// Gestion du jeu en cours d'exécution
export const setRunningGame = (gameName: string | null) => {
  runningGame = gameName;
  runningListeners.forEach((listener) => listener(runningGame));
};

export const subscribeToRunningGame = (listener: RunningGameListener) => {
  runningListeners.add(listener);
  return () => {
    runningListeners.delete(listener);
  };
};

export const getRunningGame = () => runningGame;

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

export function useRunningGame() {
  const [currentRunningGame, setCurrentRunningGame] = useState<string | null>(
    getRunningGame(),
  );

  useEffect(() => {
    return subscribeToRunningGame((gameName) => {
      setCurrentRunningGame(gameName);
    });
  }, []);

  return currentRunningGame;
}
