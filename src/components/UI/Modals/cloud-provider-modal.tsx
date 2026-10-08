import { Clock, Cloud, HardDrive, X } from "lucide-react";
import { useModalAnimation } from "../../../hooks/useModalAnimation";

interface CloudProviderModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectProvider: (provider: string) => void;
}

export default function CloudProviderModal({
  isOpen,
  onClose,
  onSelectProvider,
}: CloudProviderModalProps) {
  const { isRendered, isVisible, closeWithAnimation } = useModalAnimation(
    isOpen,
    onClose,
  );

  if (!isRendered) return null;

  const providers = [
    {
      id: "google-drive",
      name: "Google Drive",
      available: true,
      color:
        "text-blue-400 border-blue-500/20 hover:bg-blue-500/10 cursor-pointer",
    },
    {
      id: "dropbox",
      name: "Dropbox",
      available: true,
      color:
        "text-indigo-400 border-indigo-500/20 hover:bg-indigo-500/10 cursor-pointer",
    },
    {
      id: "mega",
      name: "Mega",
      available: false,
      color: "text-red-400/50 border-red-500/10 opacity-60 cursor-not-allowed",
    },
    {
      id: "proton-drive",
      name: "Proton Drive",
      available: true,
      color:
        "text-purple-400 border-purple-500/20 hover:bg-purple-500/10 cursor-pointer",
    },
  ];

  return (
    <div
      className={`fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm transition-all duration-200 ease-out ${
        isVisible ? "opacity-100" : "opacity-0 pointer-events-none"
      }`}
      onClick={closeWithAnimation}
    >
      <div
        className={`bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-sm shadow-2xl overflow-hidden flex flex-col transition-all duration-200 ease-out transform ${
          isVisible
            ? "opacity-100 scale-100 translate-y-0"
            : "opacity-0 scale-95 translate-y-3"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800">
          <h2 className="text-base font-semibold text-slate-100 flex items-center gap-2">
            <Cloud className="w-4 h-4 text-indigo-400" />
            Choisir un Cloud
          </h2>
          <button
            type="button"
            onClick={closeWithAnimation}
            className="p-1 text-slate-400 hover:text-white rounded-lg transition cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-3 flex-1 overflow-y-auto">
          <p className="text-xs text-slate-400 mb-4">
            Sélectionne le service de stockage cloud que tu souhaites connecter
            pour synchroniser tes sauvegardes:
          </p>
          {providers.map((provider) => (
            <button
              key={provider.id}
              type="button"
              disabled={!provider.available}
              onClick={() => {
                if (provider.available) {
                  onSelectProvider(provider.id);
                  closeWithAnimation();
                }
              }}
              className={`w-full flex items-center justify-between p-3 rounded-xl border bg-slate-800/50 transition-all ${provider.color}`}
            >
              <span className="text-sm font-medium text-slate-200">
                {provider.name}
              </span>
              {provider.available ? (
                <HardDrive className="w-4 h-4 text-slate-400" />
              ) : (
                <span className="flex items-center gap-1 text-[10px] text-slate-500 bg-slate-800 px-2 py-0.5 rounded-md">
                  <Clock className="w-3 h-3" />
                  Bientôt
                </span>
              )}
            </button>
          ))}
        </div>

        <div className="px-6 py-4 border-t border-slate-800 flex justify-end bg-slate-950/50">
          <button
            type="button"
            onClick={closeWithAnimation}
            className="px-4 py-2 rounded-xl text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-200 transition cursor-pointer"
          >
            Annuler
          </button>
        </div>
      </div>
    </div>
  );
}
