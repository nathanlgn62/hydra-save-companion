import { useCallback, useEffect, useState } from "react";

export interface ModalAnimationState {
  isRendered: boolean;
  isVisible: boolean;
  closeWithAnimation: () => void;
}

export function useModalAnimation(
  isOpen: boolean,
  onClose: () => void,
  duration = 200,
): ModalAnimationState {
  const [isRendered, setIsRendered] = useState(isOpen);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout>;

    if (isOpen) {
      setIsRendered(true);
      const rafId = requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          setIsVisible(true);
        });
      });
      return () => cancelAnimationFrame(rafId);
    } else {
      setIsVisible(false);
      timer = setTimeout(() => {
        setIsRendered(false);
      }, duration);
      return () => clearTimeout(timer);
    }
  }, [isOpen, duration]);

  const closeWithAnimation = useCallback(() => {
    setIsVisible(false);
    setTimeout(() => {
      onClose();
    }, duration);
  }, [onClose, duration]);

  // Support de la touche Échap avec fermeture animée
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        closeWithAnimation();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, closeWithAnimation]);

  return { isRendered, isVisible, closeWithAnimation };
}

