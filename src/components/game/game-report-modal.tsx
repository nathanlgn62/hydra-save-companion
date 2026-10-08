import { AlertTriangle, Send, X } from "lucide-react";
import { useCallback, useState } from "react";
import { useModalAnimation } from "../../hooks/useModalAnimation";

interface GameReportModalProps {
  gameTitle: string;
  gameObjectId?: string | null;
  savePath: string | null;
  isOpen: boolean;
  onClose: () => void;
}

export default function GameReportModal({
  gameTitle,
  gameObjectId,
  savePath,
  isOpen,
  onClose,
}: GameReportModalProps) {
  const [message, setMessage] = useState("");
  const [isSending, setIsSending] = useState(false);
  const [sentSuccess, setSentSuccess] = useState(false);

  const resetFormAndClose = useCallback(() => {
    setSentSuccess(false);
    setMessage("");
    onClose();
  }, [onClose]);

  const { isRendered, isVisible, closeWithAnimation } = useModalAnimation(
    isOpen,
    resetFormAndClose,
  );

  const handleSendReport = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim()) return;

    setIsSending(true);

    try {
      await fetch("TON_WEBHOOK_DISCORD_OU_API_ICI", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          game: gameTitle,
          appId: gameObjectId,
          savePath,
          message: message.trim(),
        }),
      });

      setSentSuccess(true);
      setTimeout(() => {
        closeWithAnimation();
      }, 2000);
    } catch (err) {
      console.error("Erreur lors de l'envoi du report :", err);
    } finally {
      setIsSending(false);
    }
  };

  if (!isRendered) return null;

  return (
    <div
      className={`fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm transition-all duration-200 ease-out ${
        isVisible ? "opacity-100" : "opacity-0 pointer-events-none"
      }`}
      onClick={closeWithAnimation}
    >
      <div
        className={`bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md overflow-hidden shadow-2xl flex flex-col transition-all duration-200 ease-out transform ${
          isVisible
            ? "opacity-100 scale-100 translate-y-0"
            : "opacity-0 scale-95 translate-y-3"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-800">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400">
              <AlertTriangle className="w-4 h-4" />
            </div>
            <div>
              <h3 className="font-semibold text-sm text-slate-100">
                Signaler un problème
              </h3>
              <p className="text-xs text-slate-400">{gameTitle}</p>
            </div>
          </div>
          <button
            type="button"
            onClick={closeWithAnimation}
            className="text-slate-400 hover:text-white p-1 rounded-lg hover:bg-slate-800 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {sentSuccess ? (
          <div className="p-8 text-center flex flex-col items-center gap-3">
            <div className="w-12 h-12 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center text-lg font-bold">
              ✓
            </div>
            <p className="text-sm font-medium text-slate-200">
              Message envoyé avec succès !
            </p>
            <p className="text-xs text-slate-400">
              Merci pour ton retour, le problème a bien été transmis.
            </p>
          </div>
        ) : (
          <form onSubmit={handleSendReport} className="p-5 flex flex-col gap-4">
            <div className="flex flex-col gap-1.5">
              <label className="text-xs font-medium text-slate-300">
                Décris le problème rencontré (ex: chemin de sauvegarde
                introuvable) :
              </label>
              <textarea
                rows={4}
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="Explique ce qui ne va pas..."
                className="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500 transition-colors resize-none"
                required
              />
            </div>

            <div className="flex items-center justify-end gap-2.5 pt-2 border-t border-slate-800">
              <button
                type="button"
                onClick={closeWithAnimation}
                className="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition-colors cursor-pointer"
              >
                Annuler
              </button>
              <button
                type="submit"
                disabled={isSending || !message.trim()}
                className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed text-white text-xs font-medium flex items-center gap-2 shadow-sm shadow-indigo-900/30 transition-all"
              >
                {isSending ? (
                  <>Envoi en cours...</>
                ) : (
                  <>
                    <Send className="w-3.5 h-3.5" />
                    Envoyer le rapport
                  </>
                )}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
