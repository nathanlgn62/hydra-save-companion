import {
  AlertCircle,
  CheckCircle2,
  History,
  Info,
  Trash2,
  X,
} from "lucide-react";
import { useModalAnimation } from "../../hooks/useModalAnimation";
import { useToasts } from "../../stores/toastStore";
import type { ToastType } from "../../types";

interface ToastHistoryPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

const iconMap: Record<ToastType, React.ReactNode> = {
  success: <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />,
  error: <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />,
  info: <Info className="w-4 h-4 text-indigo-400 shrink-0" />,
};

export default function ToastHistoryPanel({
  isOpen,
  onClose,
}: ToastHistoryPanelProps) {
  const { history, clearHistory } = useToasts();
  const { isRendered, isVisible, closeWithAnimation } = useModalAnimation(
    isOpen,
    onClose,
  );

  if (!isRendered) return null;

  return (
    <div className="fixed inset-0 z-50 flex justify-end overflow-hidden">
      {/* Backdrop sombre avec fondu */}
      <div
        className={`absolute inset-0 bg-slate-950/60 backdrop-blur-sm transition-opacity duration-200 ease-out ${
          isVisible ? "opacity-100" : "opacity-0 pointer-events-none"
        }`}
        onClick={closeWithAnimation}
      />

      {/* Panneau latéral avec glissement fluide */}
      <div
        className={`relative w-80 bg-slate-900 border-l border-slate-800 flex flex-col h-full shadow-2xl z-10 transition-transform duration-200 ease-out transform ${
          isVisible ? "translate-x-0" : "translate-x-full"
        }`}
      >
        {/* En-tête du panneau */}
        <div className="p-4 border-b border-slate-800 flex items-center justify-between">
          <div className="flex items-center gap-2 text-slate-100 font-medium text-sm">
            <History className="w-4 h-4 text-indigo-400" />
            <span>Historique des notifications</span>
          </div>
          <button
            type="button"
            onClick={closeWithAnimation}
            className="p-1 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Liste des notifications */}
        <div className="flex-1 overflow-y-auto p-4 flex flex-col gap-2.5">
          {history.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-slate-500 text-xs text-center gap-2">
              <History className="w-8 h-8 stroke-1" />
              <p>Aucun historique pour le moment.</p>
            </div>
          ) : (
            history.map((toast) => (
              <div
                key={toast.id}
                className="flex items-start gap-2.5 p-3 rounded-xl bg-slate-800/60 border border-slate-700/50 text-slate-200 text-xs shadow-sm"
              >
                {iconMap[toast.type]}
                <div className="flex-1 overflow-hidden">
                  <p className="break-words leading-relaxed">{toast.message}</p>
                  <span className="text-[10px] text-slate-500 mt-1 block">
                    {toast.timestamp}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Pied de page (Option vider l'historique) */}
        {history.length > 0 && (
          <div className="p-3 border-t border-slate-800 flex justify-end">
            <button
              type="button"
              onClick={clearHistory}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-rose-400 hover:bg-rose-500/10 border border-transparent hover:border-rose-500/20 transition cursor-pointer"
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span>Vider l'historique</span>
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
