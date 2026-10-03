export type ToastType = "success" | "error" | "info";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  timestamp: string;
}

let toasts: Toast[] = [];
let history: Toast[] = [];

// On cache l'objet d'état global pour éviter de recréer une référence à chaque appel
let cachedState = { toasts, history };

const listeners = new Set<() => void>();

function updateState() {
  cachedState = { toasts, history };
  listeners.forEach((l) => l());
}

export const toastStore = {
  subscribe(listener: () => void) {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },
  getState() {
    return cachedState;
  },
  show(message: string, type: ToastType = "info", duration = 4000) {
    const id = Math.random().toString(36).substring(2, 9);
    const timestamp = new Date().toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
    const newToast: Toast = { id, message, type, timestamp };

    toasts = [...toasts, newToast];
    history = [newToast, ...history].slice(0, 50);

    updateState();

    if (duration > 0) {
      setTimeout(() => {
        toastStore.dismiss(id);
      }, duration);
    }
  },
  dismiss(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
    updateState();
  },
  clearHistory() {
    history = [];
    updateState();
  },
};

import { useSyncExternalStore } from "react";

export function useToasts() {
  const state = useSyncExternalStore(
    toastStore.subscribe,
    toastStore.getState,
    toastStore.getState,
  );

  return {
    ...state,
    showToast: toastStore.show,
    dismissToast: toastStore.dismiss,
    clearHistory: toastStore.clearHistory,
  };
}
