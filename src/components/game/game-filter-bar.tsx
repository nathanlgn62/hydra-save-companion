import {
  Check,
  ChevronDown,
  Filter,
  RotateCcw,
  Search,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { SaveStatusFilter } from "../../types";

export interface StatusFilterOption {
  id: SaveStatusFilter;
  label: string;
  dotColor: string;
}

export const STATUS_FILTER_OPTIONS: StatusFilterOption[] = [
  { id: "all", label: "Tous les statuts", dotColor: "bg-slate-400" },
  { id: "up-to-date", label: "À jour", dotColor: "bg-emerald-400" },
  { id: "local-newer", label: "Local plus récent", dotColor: "bg-indigo-400" },
  { id: "cloud-newer", label: "Cloud plus récent", dotColor: "bg-amber-400" },
  { id: "to-download", label: "À télécharger", dotColor: "bg-sky-400" },
  { id: "never-synced", label: "Jamais synchronisé", dotColor: "bg-amber-500" },
  { id: "no-save", label: "Sans sauvegarde", dotColor: "bg-slate-500" },
];

interface GameFilterBarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  statusFilter: SaveStatusFilter;
  onStatusFilterChange: (status: SaveStatusFilter) => void;
  totalGames: number;
  filteredCount: number;
  statusCounts: Record<SaveStatusFilter, number>;
  onReset: () => void;
}

export default function GameFilterBar({
  searchQuery,
  onSearchChange,
  statusFilter,
  onStatusFilterChange,
  totalGames,
  filteredCount,
  statusCounts,
  onReset,
}: GameFilterBarProps) {
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const selectedOption =
    STATUS_FILTER_OPTIONS.find((opt) => opt.id === statusFilter) ||
    STATUS_FILTER_OPTIONS[0];

  const hasActiveFilters = searchQuery.trim().length > 0 || statusFilter !== "all";

  // Fermer le dropdown en cliquant à l'extérieur
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsDropdownOpen(false);
      }
    }

    if (isDropdownOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isDropdownOpen]);

  return (
    <div className="w-full mb-4 flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2.5">
      {/* Barre de recherche */}
      <div className="relative flex-1 max-w-md">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              onSearchChange("");
            }
          }}
          placeholder="Rechercher un jeu par titre..."
          className="w-full h-9 pl-9 pr-8 bg-slate-900/90 border border-slate-800 hover:border-slate-700 focus:border-indigo-500 rounded-xl text-xs text-slate-100 placeholder:text-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500 transition-all"
        />
        {searchQuery && (
          <button
            type="button"
            onClick={() => onSearchChange("")}
            className="absolute right-2.5 top-1/2 -translate-y-1/2 p-0.5 rounded-md text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition"
            title="Effacer la recherche"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        )}
      </div>

      {/* Filtres de statut et actions */}
      <div className="flex items-center gap-2 flex-wrap">
        {/* Menu déroulant de filtre par statut */}
        <div className="relative" ref={dropdownRef}>
          <button
            type="button"
            onClick={() => setIsDropdownOpen((prev) => !prev)}
            className={`h-9 px-3 rounded-xl border text-xs font-medium flex items-center gap-2 transition-all cursor-pointer shadow-sm ${
              statusFilter !== "all"
                ? "bg-indigo-600/10 border-indigo-500/40 text-indigo-300 hover:bg-indigo-600/20"
                : "bg-slate-900/90 border-slate-800 hover:border-slate-700 text-slate-300 hover:text-slate-100"
            }`}
          >
            <Filter className="w-3.5 h-3.5 text-slate-400 shrink-0" />
            <div className="flex items-center gap-1.5 truncate">
              {statusFilter !== "all" && (
                <span
                  className={`w-2 h-2 rounded-full shrink-0 ${selectedOption.dotColor}`}
                />
              )}
              <span className="truncate">{selectedOption.label}</span>
            </div>
            <span className="px-1.5 py-0.5 rounded-full bg-slate-800 border border-slate-700/60 text-[10px] text-slate-300 font-mono ml-0.5">
              {statusCounts[statusFilter] ?? 0}
            </span>
            <ChevronDown
              className={`w-3.5 h-3.5 text-slate-400 transition-transform duration-200 shrink-0 ${
                isDropdownOpen ? "rotate-180" : ""
              }`}
            />
          </button>

          {isDropdownOpen && (
            <div className="absolute right-0 top-full mt-1.5 w-64 bg-slate-900 border border-slate-800 rounded-xl shadow-2xl p-1 z-50 animate-in fade-in zoom-in-95 duration-100">
              <div className="px-2.5 py-1.5 text-[10px] font-semibold text-slate-400 uppercase tracking-wider border-b border-slate-800/80 mb-1">
                Filtrer par statut
              </div>
              <div className="flex flex-col gap-0.5">
                {STATUS_FILTER_OPTIONS.map((opt) => {
                  const isSelected = statusFilter === opt.id;
                  const count = statusCounts[opt.id] ?? 0;

                  return (
                    <button
                      key={opt.id}
                      type="button"
                      onClick={() => {
                        onStatusFilterChange(opt.id);
                        setIsDropdownOpen(false);
                      }}
                      className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer ${
                        isSelected
                          ? "bg-indigo-600/20 text-indigo-300 font-medium"
                          : "text-slate-300 hover:bg-slate-800/70 hover:text-white"
                      }`}
                    >
                      <div className="flex items-center gap-2 truncate">
                        <span
                          className={`w-2 h-2 rounded-full shrink-0 ${opt.dotColor}`}
                        />
                        <span className="truncate">{opt.label}</span>
                      </div>
                      <div className="flex items-center gap-1.5 shrink-0 ml-2">
                        <span
                          className={`px-1.5 py-0.2 rounded-full text-[10px] font-mono ${
                            isSelected
                              ? "bg-indigo-500/20 text-indigo-300"
                              : "bg-slate-800 text-slate-400"
                          }`}
                        >
                          {count}
                        </span>
                        {isSelected && (
                          <Check className="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                        )}
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>
          )}
        </div>

        {/* Bouton de réinitialisation si des filtres sont actifs */}
        {hasActiveFilters && (
          <button
            type="button"
            onClick={onReset}
            className="h-9 px-2.5 rounded-xl bg-slate-800/70 hover:bg-slate-800 border border-slate-700/60 text-slate-300 hover:text-white text-xs flex items-center gap-1.5 transition cursor-pointer shadow-sm"
            title="Réinitialiser tous les filtres"
          >
            <RotateCcw className="w-3.5 h-3.5 text-slate-400" />
            <span className="hidden sm:inline">Réinitialiser</span>
          </button>
        )}

        {/* Compteur de résultats */}
        <div className="text-[11px] text-slate-400 font-mono pl-1 hidden md:block">
          {hasActiveFilters ? (
            <span>
              <span className="text-slate-200 font-semibold">{filteredCount}</span>/
              {totalGames} {totalGames > 1 ? "jeux" : "jeu"}
            </span>
          ) : (
            <span>
              <span className="text-slate-200 font-semibold">{totalGames}</span>{" "}
              {totalGames > 1 ? "jeux" : "jeu"}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

