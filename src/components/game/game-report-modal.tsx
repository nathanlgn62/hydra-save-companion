import { AlertTriangle, Check, Folder, Mail, Send, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useModalAnimation } from "../../hooks/use-modal-animation";

interface GameReportModalProps {
  gameTitle: string;
  gameObjectId?: string | null;
  savePath: string | null;
  isOpen: boolean;
  onClose: () => void;
}

const REPORT_API_URL =
  import.meta.env.VITE_REPORT_API_URL ||
  "https://nathandev.fr/api/hydra-save-companion/report";

const APP_VERSION = "0.1.0";

const ISSUE_CATEGORIES = [
  { id: "path_not_found", label: "Path not found" },
  { id: "sync_failure", label: "Sync failure" },
  { id: "save_corrupted", label: "Corrupted save" },
  { id: "other", label: "Other issue" },
] as const;

type IssueCategoryId = (typeof ISSUE_CATEGORIES)[number]["id"];

function detectPlatform(): string {
  if (typeof navigator === "undefined") return "PC";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) return "Windows";
  if (ua.includes("linux")) return "Linux";
  if (ua.includes("mac")) return "macOS";
  return navigator.platform || "PC";
}

export default function GameReportModal({
  gameTitle,
  gameObjectId,
  savePath,
  isOpen,
  onClose,
}: GameReportModalProps) {
  const [category, setCategory] = useState<IssueCategoryId>("path_not_found");
  const [customSavePath, setCustomSavePath] = useState(savePath || "");
  const [message, setMessage] = useState("");
  const [userContact, setUserContact] = useState("");
  const [isSending, setIsSending] = useState(false);
  const [sentSuccess, setSentSuccess] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Synchronise le chemin quand le modal s'ouvre pour un jeu donné
  useEffect(() => {
    if (isOpen) {
      setCustomSavePath(savePath || "");
    }
  }, [isOpen, savePath]);

  const resetFormAndClose = useCallback(() => {
    setSentSuccess(false);
    setErrorMessage(null);
    setMessage("");
    setUserContact("");
    setCategory("path_not_found");
    setCustomSavePath(savePath || "");
    onClose();
  }, [onClose, savePath]);

  const { isRendered, isVisible, closeWithAnimation } = useModalAnimation(
    isOpen,
    resetFormAndClose,
  );

  const handleSendReport = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim()) return;

    setIsSending(true);
    setErrorMessage(null);

    const platform = detectPlatform();
    const selectedCategory =
      ISSUE_CATEGORIES.find((c) => c.id === category)?.label || category;

    try {
      const response = await fetch(REPORT_API_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          game: gameTitle,
          appId: gameObjectId || null,
          category: selectedCategory,
          savePath: customSavePath.trim() || savePath || null,
          message: message.trim(),
          appVersion: APP_VERSION,
          platform,
          userContact: userContact.trim() || null,
        }),
      });

      if (!response.ok) {
        throw new Error(`Erreur serveur (${response.status})`);
      }

      setSentSuccess(true);
      setTimeout(() => {
        closeWithAnimation();
      }, 2000);
    } catch (err) {
      console.error("Error submitting report:", err);
      setErrorMessage(
        "Unable to send report. Please check your internet connection or try again later.",
      );
    } finally {
      setIsSending(false);
    }
  };

  if (!isRendered) return null;

  const currentPlatform = detectPlatform();

  return (
    <div
      className={`fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm transition-all duration-200 ease-out ${
        isVisible ? "opacity-100" : "opacity-0 pointer-events-none"
      }`}
      onClick={closeWithAnimation}
    >
      <div
        className={`bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-lg max-h-[92vh] overflow-hidden shadow-2xl flex flex-col transition-all duration-200 ease-out transform ${
          isVisible
            ? "opacity-100 scale-100 translate-y-0"
            : "opacity-0 scale-95 translate-y-3"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-800 bg-slate-950/40">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400 shrink-0">
              <AlertTriangle className="w-4 h-4" />
            </div>
            <div>
              <h3 className="font-semibold text-sm text-slate-100">
                Report an Issue
              </h3>
              <div className="flex items-center gap-2 mt-0.5">
                <span className="text-xs text-slate-400 truncate max-w-[200px]">
                  {gameTitle}
                </span>
                {gameObjectId && (
                  <span className="px-1.5 py-0.2 rounded bg-slate-800 text-[10px] font-mono text-slate-400">
                    #{gameObjectId}
                  </span>
                )}
              </div>
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
          <div className="p-10 text-center flex flex-col items-center gap-3">
            <div className="w-14 h-14 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center text-xl font-bold">
              <Check className="w-6 h-6" />
            </div>
            <p className="text-base font-medium text-slate-100">
              Report sent successfully!
            </p>
            <p className="text-xs text-slate-400 max-w-xs">
              Thank you for your feedback. Technical details have been transmitted to the maintainers.
            </p>
          </div>
        ) : (
          <form
            onSubmit={handleSendReport}
            className="p-5 flex flex-col gap-4 overflow-y-auto"
          >
            {errorMessage && (
              <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-xs text-red-400">
                {errorMessage}
              </div>
            )}

            {/* Issue Category */}
            <div className="flex flex-col gap-1.5">
              <label className="text-xs font-medium text-slate-300">
                Issue Type:
              </label>
              <div className="grid grid-cols-2 gap-2">
                {ISSUE_CATEGORIES.map((item) => {
                  const isSelected = category === item.id;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      onClick={() => setCategory(item.id)}
                      className={`px-3 py-2 rounded-xl text-xs font-medium text-left border transition-all cursor-pointer ${
                        isSelected
                          ? "bg-indigo-600/20 border-indigo-500 text-indigo-200 shadow-sm shadow-indigo-900/30"
                          : "bg-slate-950/60 border-slate-800 text-slate-400 hover:border-slate-700 hover:text-slate-300"
                      }`}
                    >
                      {item.label}
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Save Directory */}
            <div className="flex flex-col gap-1.5">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium text-slate-300 flex items-center gap-1.5">
                  <Folder className="w-3.5 h-3.5 text-slate-400" />
                  Save Directory:
                </label>
                <span className="text-[10px] text-slate-500">
                  Editable if incorrect
                </span>
              </div>
              <input
                type="text"
                value={customSavePath}
                onChange={(e) => setCustomSavePath(e.target.value)}
                placeholder="Path not found (you can provide the actual save path here)"
                className="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs font-mono text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500 transition-colors"
              />
            </div>

            {/* Detailed Description */}
            <div className="flex flex-col gap-1.5">
              <label className="text-xs font-medium text-slate-300">
                Issue Description: <span className="text-red-400">*</span>
              </label>
              <textarea
                rows={3}
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="Describe what is wrong (e.g., folder is empty, save doesn't upload to Google Drive...)"
                className="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500 transition-colors resize-none"
                required
              />
            </div>

            {/* Optional contact email */}
            <div className="flex flex-col gap-1.5">
              <label className="text-xs font-medium text-slate-300 flex items-center gap-1.5">
                <Mail className="w-3.5 h-3.5 text-slate-400" />
                Contact Email <span className="text-slate-500 font-normal">(optional):</span>
              </label>
              <input
                type="email"
                value={userContact}
                onChange={(e) => setUserContact(e.target.value)}
                placeholder="To receive updates when the issue is resolved"
                className="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-slate-200 placeholder:text-slate-600 focus:outline-none focus:border-indigo-500 transition-colors"
              />
            </div>

            {/* Footer buttons */}
            <div className="pt-3 border-t border-slate-800 flex flex-col gap-3">
              <div className="flex items-center justify-between text-[11px] text-slate-500">
                <span>
                  Hydra Save Companion <span className="font-mono">v{APP_VERSION}</span>
                </span>
                <span className="font-mono">OS: {currentPlatform}</span>
              </div>

              <div className="flex items-center justify-end gap-2.5">
                <button
                  type="button"
                  onClick={closeWithAnimation}
                  className="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition-colors cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isSending || !message.trim()}
                  className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed text-white text-xs font-medium flex items-center gap-2 shadow-sm shadow-indigo-900/30 transition-all cursor-pointer"
                >
                  {isSending ? (
                    <>Sending...</>
                  ) : (
                    <>
                      <Send className="w-3.5 h-3.5" />
                      Submit Report
                    </>
                  )}
                </button>
              </div>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
