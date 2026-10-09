import { SyncBadgeConfig, SyncBadgeVariant } from "../../types";

const variantStyles: Record<SyncBadgeVariant, string> = {
  neutral: "bg-slate-800/80 border-slate-700/60 text-slate-400",
  warning: "bg-amber-500/10 border-amber-500/20 text-amber-400",
  success: "bg-emerald-500/10 border-emerald-500/20 text-emerald-400",
  info: "bg-indigo-500/10 border-indigo-500/20 text-indigo-400",
  danger: "bg-rose-500/10 border-rose-500/20 text-rose-400",
};

export default function SyncBadge({ config }: { config: SyncBadgeConfig }) {
  const { label, variant, Icon, description } = config;

  return (
    <div className="relative group inline-flex items-center">
      {/* Le Badge */}
      <div
        className={`flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-[10px] font-medium backdrop-blur-md cursor-help ${variantStyles[variant]}`}
      >
        <Icon className="w-3 h-3" />
        <span>{label}</span>
      </div>

      {/* L'infobulle un peu plus compacte (w-56) alignée à droite pour rester dans la carte */}
      {description && (
        <div className="absolute top-full right-0 mt-2 hidden group-hover:block z-50 w-52 p-2 text-[11px] text-slate-200 bg-slate-900/95 border border-slate-700/80 rounded-lg shadow-xl backdrop-blur-md text-center pointer-events-none transition-all">
          {description}
          {/* Petite flèche en haut, calée vers la droite pour pointer sur le badge */}
          <div className="absolute bottom-full right-3 -mb-px border-4 border-transparent border-b-slate-800" />
        </div>
      )}
    </div>
  );
}
