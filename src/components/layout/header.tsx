import { Cloud, CloudOff, History, Settings } from "lucide-react";
import { useState } from "react";
import { useSettings } from "../../hooks/use-settings";
import { loginCloud, useCloudStatus } from "../../stores/cloud-store";
import { getProviderDisplayName } from "../../utils/provider-display-name";
import SettingsModal from "../setting/setting-modal";
import ToastHistoryPanel from "../toast/toast-history-pannel";
import CloudProviderModal from "../cloud/cloud-provider-modal";
import logoSvg from "../../assets/logo.svg";

export default function Header() {
  const [loading] = useState<boolean>(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState<boolean>(false);
  const [isCloudModalOpen, setIsCloudModalOpen] = useState<boolean>(false);

  const { settings, updateSettings, loading: loadingSettings } = useSettings();
  const { isDriveConnected, cloudProvider, disconnectGoogle } =
    useCloudStatus();

  const handleCloudAction = () => {
    if (isDriveConnected) {
      if (confirm("Do you want to disconnect from Cloud storage?")) {
        disconnectGoogle();
      }
    } else {
      setIsCloudModalOpen(true);
    }
  };

  const handleSelectProvider = (providerId: string) => {
    loginCloud(providerId);
  };

  return (
    <>
      <header className="h-14 bg-slate-900 border-b border-slate-800 px-4 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <div className="relative w-8 h-8 rounded-lg p-0.5 bg-[#1A0731] border border-cyan-500/30 shadow-md shadow-cyan-950/40 overflow-hidden flex items-center justify-center shrink-0">
            <img
              src={logoSvg}
              alt="Hydra Save Companion"
              className="w-full h-full object-cover"
            />
          </div>

          {settings?.demoMode && (
            <span
              onClick={() => updateSettings({ demoMode: false })}
              className="cursor-pointer text-[10px] font-semibold tracking-wide uppercase px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/30 hover:bg-amber-500/20 transition"
              title="Demo Mode active (click to disable)"
            >
              Demo Mode
            </span>
          )}
        </div>

        <div className="flex items-center gap-3">

          <button
            type="button"
            onClick={handleCloudAction}
            disabled={loading}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-medium border transition-all cursor-pointer disabled:opacity-50 ${
              isDriveConnected
                ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/20"
                : "bg-rose-500/10 border-rose-500/20 text-rose-400 hover:bg-rose-500/20"
            }`}
            title={
              isDriveConnected
                ? `Connected to ${getProviderDisplayName(cloudProvider)} (Click to disconnect)`
                : "Choose a Cloud service"
            }
          >
            {isDriveConnected ? (
              <Cloud className="w-3.5 h-3.5" />
            ) : (
              <CloudOff className="w-3.5 h-3.5" />
            )}
            <span>
              {loading
                ? "Connecting..."
                : isDriveConnected
                  ? getProviderDisplayName(cloudProvider)
                  : "Cloud Disconnected"}
            </span>
          </button>

          <button
            type="button"
            onClick={() => setIsHistoryOpen(true)}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition cursor-pointer"
            title="Notification History"
          >
            <History className="w-4 h-4" />
          </button>

          <button
            type="button"
            onClick={() => setIsSettingsOpen(true)}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition cursor-pointer"
            title="Settings"
          >
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </header>

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        settings={settings}
        updateSettings={updateSettings}
        loadingSettings={loadingSettings}
      />

      <CloudProviderModal
        isOpen={isCloudModalOpen}
        onClose={() => setIsCloudModalOpen(false)}
        onSelectProvider={handleSelectProvider}
      />

      <ToastHistoryPanel
        isOpen={isHistoryOpen}
        onClose={() => setIsHistoryOpen(false)}
      />
    </>
  );
}
