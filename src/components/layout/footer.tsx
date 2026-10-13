import { Cloud, CloudOff, Loader2 } from "lucide-react";
import { getRunningGame } from "../../stores/game-store";
import { useCloudStatus } from "../../stores/cloud-store";
import { useCloudStorage } from "../../hooks/use-cloud-storage";
import { getProviderDisplayName } from "../../utils/provider-display-name";

export default function Footer() {
  const runningGame = getRunningGame();
  const { isDriveConnected, cloudProvider } = useCloudStatus();
  const { data: storageInfo, isLoading } = useCloudStorage();

  const usagePercent =
    storageInfo?.totalBytes && storageInfo.totalBytes > 0
      ? Math.min(100, Math.round((storageInfo.usedBytes / storageInfo.totalBytes) * 100))
      : null;

  return (
    <footer className="h-8 bg-slate-950 border-t border-slate-800/80 px-3 flex items-center justify-between text-[11px] text-slate-500 shrink-0 select-none">
      <div className="flex items-center gap-2">
        {runningGame ? (
          <>
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="text-slate-300 font-medium">
              Current Game:{" "}
              <span className="text-emerald-400">{runningGame}</span>
            </span>
          </>
        ) : (
          <>
            <span className="w-2 h-2 rounded-full bg-slate-600" />
            <span>No game running</span>
          </>
        )}
      </div>

      <div className="flex items-center">
        {!isDriveConnected ? (
          <div className="flex items-center gap-1.5 text-slate-500">
            <CloudOff className="w-3.5 h-3.5 text-slate-600" />
            <span>Cloud disconnected</span>
          </div>
        ) : isLoading ? (
          <div className="flex items-center gap-1.5 text-slate-400">
            <Loader2 className="w-3 h-3 animate-spin text-slate-500" />
            <span>{getProviderDisplayName(cloudProvider)}: Loading...</span>
          </div>
        ) : storageInfo ? (
          <div
            className="flex items-center gap-2.5 text-slate-400"
            title={storageInfo.freeFormatted ? `${storageInfo.freeFormatted} available storage` : undefined}
          >
            <div className="flex items-center gap-1.5">
              <Cloud className="w-3.5 h-3.5 text-emerald-400" />
              <span className="font-medium text-slate-300">
                {storageInfo.providerName}:
              </span>
              <span>
                {storageInfo.usedFormatted}
                {storageInfo.totalFormatted ? ` / ${storageInfo.totalFormatted}` : " used"}
              </span>
            </div>

            {usagePercent !== null && (
              <div className="hidden sm:flex items-center gap-1.5">
                <div className="w-14 h-1.5 bg-slate-800/90 rounded-full overflow-hidden p-px">
                  <div
                    className={`h-full rounded-full transition-all duration-300 ${
                      usagePercent > 90
                        ? "bg-rose-500"
                        : usagePercent > 75
                          ? "bg-amber-400"
                          : "bg-emerald-400"
                    }`}
                    style={{ width: `${usagePercent}%` }}
                  />
                </div>
                {storageInfo.freeFormatted && (
                  <span className="text-slate-500 text-[10px]">
                    ({storageInfo.freeFormatted} free)
                  </span>
                )}
              </div>
            )}
          </div>
        ) : (
          <div className="flex items-center gap-1.5 text-slate-400">
            <Cloud className="w-3.5 h-3.5 text-emerald-400" />
            <span>{getProviderDisplayName(cloudProvider)}: Connected</span>
          </div>
        )}
      </div>
    </footer>
  );
}
