export type ToastType = "success" | "error" | "info";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
}

let toasts: Toast[] = [];
const listeners = new Set<() => void>();

export const toastStore = {
  subscribe(listener: () => void) {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },
  getState() {
    return toasts;
  },
  show(message: string, type: ToastType = "info", duration = 4000) {
    const id = Math.random().toString(36).substring(2, 9);
    toasts = [...toasts, { id, message, type }];
    listeners.forEach((l) => l());

    if (duration > 0) {
      setTimeout(() => {
        toastStore.dismiss(id);
      }, duration);
    }
  },
  dismiss(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
    listeners.forEach((l) => l());
  },
};

// Hook React simple pour consommer le store
import { useSyncExternalStore } from "react";

export function useToasts() {
  const currentToasts = useSyncExternalStore(
    toastStore.subscribe,
    toastStore.getState,
    toastStore.getState,
  );

  return {
    toasts: currentToasts,
    showToast: toastStore.show,
    dismissToast: toastStore.dismiss,
  };
}
