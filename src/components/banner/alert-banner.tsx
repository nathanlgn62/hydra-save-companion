export default function AlertBanner() {
  return (
    <div className="w-full flex items-start gap-3 p-3.5 bg-red-500/10 border border-red-500/20 text-red-400 rounded-xl text-sm leading-relaxed backdrop-blur-sm shrink-0 mb-4">
      <svg
        className="w-5 h-5 shrink-0 mt-0.5 text-red-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
        />
      </svg>
      <div>
        <span className="font-semibold block mb-0.5 text-red-300">
          Jeux non pris en charge
        </span>
        Seuls les jeux présents dans Hydra Launcher et installés sont affichés.
        Les jeux importés des boutiques officielles disposent déjà de leur
        propre sauvegarde cloud.
      </div>
    </div>
  );
}
