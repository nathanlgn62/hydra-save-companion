import { useEffect, useState } from "react";
import logoSvg from "../../assets/logo.svg";

interface SplashScreenProps {
  isLoading: boolean;
  onFinish?: () => void;
}

export default function SplashScreen({
  isLoading,
  onFinish,
}: SplashScreenProps) {
  const [progress, setProgress] = useState(0);
  const [isExiting, setIsExiting] = useState(false);
  const [isVisible, setIsVisible] = useState(true);

  // Simulation fluide et progressive de la progression
  useEffect(() => {
    let timer: ReturnType<typeof setInterval> | null = null;

    if (isLoading) {
      timer = setInterval(() => {
        setProgress((prev) => {
          // Ralentit naturellement au fur et à mesure qu'on approche de 92%
          if (prev < 30) return prev + Math.random() * 4 + 2;
          if (prev < 65) return prev + Math.random() * 2 + 1;
          if (prev < 90) return prev + Math.random() * 0.8 + 0.3;
          if (prev < 94) return prev + 0.2;
          return prev;
        });
      }, 50);
    } else {
      // Les données sont chargées : on propulse la barre à 100% de manière fluide
      setProgress(100);

      // Court délai pour apprécier l'état complété (100%), puis fondu de sortie
      const exitTimer = setTimeout(() => {
        setIsExiting(true);
      }, 300);

      // Démontage complet après la transition d'opacité
      const unmountTimer = setTimeout(() => {
        setIsVisible(false);
        onFinish?.();
      }, 950);

      return () => {
        clearTimeout(exitTimer);
        clearTimeout(unmountTimer);
      };
    }

    return () => {
      if (timer) clearInterval(timer);
    };
  }, [isLoading, onFinish]);

  if (!isVisible) return null;

  // Libellé d'état dynamique selon la progression
  const getStatusLabel = () => {
    if (progress >= 100) return "Compagnon prêt !";
    if (progress >= 85) return "Finalisation de la vérification...";
    if (progress >= 60) return "Vérification des sauvegardes et du Cloud...";
    if (progress >= 30) return "Détection des jeux installés...";
    return "Initialisation du compagnon...";
  };

  const clampedProgress = Math.min(Math.max(Math.round(progress), 0), 100);

  return (
    <div
      className={`fixed inset-0 z-50 flex flex-col items-center justify-center bg-[#090514] select-none transition-all duration-700 ease-out ${
        isExiting
          ? "opacity-0 scale-105 pointer-events-none filter blur-sm"
          : "opacity-100 scale-100"
      }`}
    >
      {/* Halo radial d'ambiance avec résonance chromatique du logo */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_80%_60%_at_50%_40%,rgba(47,172,246,0.14),rgba(19,255,182,0.08)_35%,transparent_70%)] pointer-events-none" />

      {/* Grille technique d'arrière-plan */}
      <div className="absolute inset-0 bg-[linear-gradient(to_right,rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(to_bottom,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:3.5rem_3.5rem] pointer-events-none" />

      <div className="relative z-10 flex flex-col items-center max-w-sm px-6 text-center">
        {/* Conteneur Logo avec pulsation douce et liseré cyan */}
        <div className="relative mb-6">
          <div className="absolute -inset-4 rounded-3xl bg-gradient-to-tr from-[#13FFB6]/25 via-[#25D7CF]/20 to-[#2FACF6]/25 blur-xl opacity-80 animate-pulse" />
          <div className="relative w-24 h-24 rounded-2xl p-1 bg-[#1A0731] border border-cyan-400/30 shadow-2xl shadow-cyan-950/70 overflow-hidden flex items-center justify-center transform transition duration-500 hover:scale-105">
            <img
              src={logoSvg}
              alt="Hydra Save Companion"
              className="w-full h-full object-cover select-none"
            />
          </div>
        </div>

        {/* Titre & Signature de marque */}
        <h1 className="text-xl font-bold tracking-tight text-white mb-1 flex items-center gap-2">
          <span>Hydra</span>
          <span className="text-xs font-semibold px-2 py-0.5 rounded-md bg-gradient-to-r from-emerald-400/20 to-cyan-400/20 text-cyan-300 border border-cyan-400/30 tracking-widest uppercase">
            Save Companion
          </span>
        </h1>
        <p className="text-xs text-slate-400 font-medium tracking-wide mb-8">
          Gestion & synchronisation universelle de sauvegardes
        </p>

        {/* Barre de progression fluide */}
        <div className="w-64 h-1.5 bg-slate-900/90 rounded-full overflow-hidden border border-slate-800/80 p-0.5 mb-3 relative shadow-inner">
          <div
            className="h-full bg-gradient-to-r from-[#13FFB6] via-[#25D7CF] to-[#2FACF6] rounded-full transition-[width] duration-300 ease-out relative shadow-[0_0_12px_rgba(37,215,207,0.7)]"
            style={{ width: `${clampedProgress}%` }}
          >
            {/* Tête de lecture lumineuse */}
            <div className="absolute right-0 top-1/2 -translate-y-1/2 w-2 h-2 rounded-full bg-white shadow-[0_0_8px_#ffffff]" />
          </div>
        </div>

        {/* Indicateurs d'état et pourcentage */}
        <div className="w-64 flex items-center justify-between text-[11px] font-mono text-slate-400 px-0.5">
          <span className="truncate pr-2">{getStatusLabel()}</span>
          <span className="text-cyan-400 font-semibold tabular-nums shrink-0">
            {clampedProgress}%
          </span>
        </div>
      </div>
    </div>
  );
}
