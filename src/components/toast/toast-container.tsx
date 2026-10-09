import { AlertCircle, CheckCircle2, Info, X } from "lucide-react";
import type { ReactElement } from "react";
import { ToastType } from "../../types";
import { useToasts } from "../../stores/toast-store";

const iconMap: Record<ToastType, ReactElement> = {
  success: <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />,
  error: <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />,
  info: <Info className="w-4 h-4 text-indigo-400 shrink-0" />,
};

const borderMap: Record<ToastType, string> = {
  success: "border-emerald-500/20 bg-emerald-500/10 text-emerald-200",
  error: "border-rose-500/20 bg-rose-500/10 text-rose-200",
  info: "border-slate-700/60 bg-slate-900/90 text-slate-200",
};

export default function ToastContainer() {
  const { toasts, dismissToast } = useToasts();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm w-full pointer-events-none">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`pointer-events-auto flex items-center justify-between gap-3 px-4 py-3 rounded-xl border backdrop-blur-md shadow-xl transition-all animate-in fade-in slide-in-from-bottom-3 duration-200 ${
            borderMap[toast.type]
          }`}
        >
          <div className="flex items-center gap-2.5 overflow-hidden">
            {iconMap[toast.type]}
            <p className="text-xs font-medium truncate">{toast.message}</p>
          </div>
          <button
            type="button"
            onClick={() => dismissToast(toast.id)}
            className="text-slate-400 hover:text-white transition cursor-pointer p-0.5"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      ))}
    </div>
  );
}
