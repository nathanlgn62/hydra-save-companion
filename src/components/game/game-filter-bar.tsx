import {
  Check,
  ChevronDown,
  Filter,
  RefreshCw,
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
  onRefresh?: () => void;
  isRefreshing?: boolean;
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
  onRefresh,
  isRefreshing = false,
}: GameFilterBarProps) {
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const selectedOption =
    STATUS_FILTER_OPTIONS.find((opt) => opt.id === statusFilter) ||
    STATUS_FILTER_OPTIONS[0];

  const hasActiveFilters = searchQuery.trim().length > 0 || statusFilter !== "all";

  // Raccourci clavier global (Ctrl+K, Cmd+K ou '/') pour focuser la recherche
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      // Ignorer si l'utilisateur est déjà en train de taper dans un champ sauf si c'est Escape
      const target = event.target as HTMLElement;
      const isInputFocused =
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable;

      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
        return;
      }

      if (event.key === "/" && !isInputFocused) {
        event.preventDefault();
        inputRef.current?.focus();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

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
    <div className="fixed bottom-11 left-1/2 -translate-x-1/2 z-40 max-w-[calc(100vw-2rem)] pointer-events-auto">
      {/* Container Dock / Capsule Flottante */}
      <div className="flex items-center gap-1.5 p-1.5 rounded-2xl bg-slate-900/85 backdrop-blur-xl border border-slate-700/60 shadow-[0_12px_36px_rgba(0,0,0,0.55),0_0_0_1px_rgba(255,255,255,0.06)] ring-1 ring-black/40 transition-all duration-300">
        {/* Champ de recherche compact et réactif */}
        <div className="relative flex items-center">
          <Search className="absolute left-3 w-3.5 h-3.5 text-slate-400 pointer-events-none transition-colors" />
          <input
            ref={inputRef}
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                if (searchQuery) {
                  onSearchChange("");
                } else {
                  inputRef.current?.blur();
                }
              }
            }}
            placeholder="Rechercher..."
            className="w-40 sm:w-60 h-8 pl-8.5 pr-14 bg-slate-950/70 hover:bg-slate-950/90 focus:bg-slate-950 border border-slate-800 focus:border-indigo-500/80 rounded-xl text-xs text-slate-100 placeholder:text-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500/50 transition-all duration-200"
          />

          {/* Badge indicateur de raccourci clavier ou bouton effacer */}
          {searchQuery ? (
            <button
              type="button"
              onClick={() => {
                onSearchChange("");
                inputRef.current?.focus();
              }}
              className="absolute right-2 p-0.5 rounded text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition cursor-pointer"
              title="Effacer la recherche (Échap)"
            >
              <X className="w-3 h-3" />
            </button>
          ) : (
            <div className="absolute right-2 flex items-center gap-0.5 pointer-events-none select-none">
              <kbd className="px-1.5 py-0.5 text-[9px] font-mono font-medium text-slate-400 bg-slate-800/80 border border-slate-700/60 rounded shadow-xs">
                ⌘K
              </kbd>
            </div>
          )}
        </div>

        {/* Séparateur vertical subtil */}
        <div className="w-[1px] h-5 bg-slate-800/90 mx-0.5" />

        {/* Menu déroulant de filtre par statut */}
        <div className="relative" ref={dropdownRef}>
          <button
            type="button"
            onClick={() => setIsDropdownOpen((prev) => !prev)}
            className={`h-8 px-2.5 rounded-xl border text-xs font-medium flex items-center gap-2 transition-all duration-200 cursor-pointer ${
              statusFilter !== "all"
                ? "bg-indigo-600/15 border-indigo-500/50 text-indigo-200 hover:bg-indigo-600/25 shadow-sm shadow-indigo-950/40"
                : "bg-slate-950/70 hover:bg-slate-950/90 border-slate-800 hover:border-slate-700/80 text-slate-300 hover:text-slate-100"
            }`}
          >
            <Filter className="w-3.5 h-3.5 text-slate-400 shrink-0" />
            <div className="flex items-center gap-1.5 truncate max-w-[110px] sm:max-w-none">
              {statusFilter !== "all" && (
                <span
                  className={`w-1.5 h-1.5 rounded-full shrink-0 ${selectedOption.dotColor}`}
                />
              )}
              <span className="truncate">{selectedOption.label}</span>
            </div>
            <span className="px-1.5 py-0.2 rounded-md bg-slate-800/90 border border-slate-700/50 text-[10px] text-slate-300 font-mono shrink-0">
              {statusCounts[statusFilter] ?? 0}
            </span>
            <ChevronDown
              className={`w-3.5 h-3.5 text-slate-400 transition-transform duration-200 shrink-0 ${
                isDropdownOpen ? "rotate-180" : ""
              }`}
            />
          </button>

          {/* Popover vers le HAUT (bottom-full) car nous sommes ancrés en bas */}
          {isDropdownOpen && (
            <div className="absolute left-1/2 -translate-x-1/2 sm:left-auto sm:right-0 sm:translate-x-0 bottom-full mb-2.5 w-64 bg-slate-900/95 backdrop-blur-xl border border-slate-700/70 rounded-2xl shadow-2xl p-1.5 z-50 animate-in fade-in zoom-in-95 duration-150 ring-1 ring-black/50">
              <div className="px-2.5 py-1.5 text-[10px] font-semibold text-slate-400 uppercase tracking-wider border-b border-slate-800/80 mb-1 flex items-center justify-between">
                <span>Statut de sauvegarde</span>
                <span className="text-slate-500 font-mono text-[9px] lowercase font-normal">
                  {totalGames} jeux
                </span>
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
                      className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-xl text-xs transition-colors cursor-pointer ${
                        isSelected
                          ? "bg-indigo-600/25 text-indigo-200 font-medium border border-indigo-500/30"
                          : "text-slate-300 hover:bg-slate-800/80 hover:text-white border border-transparent"
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
                              ? "bg-indigo-500/30 text-indigo-200 font-semibold"
                              : "bg-slate-800/90 text-slate-400"
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

        {/* Bouton de réinitialisation si actif */}
        {hasActiveFilters && (
          <button
            type="button"
            onClick={onReset}
            className="h-8 px-2.5 rounded-xl bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30 text-amber-300 hover:text-amber-200 text-xs flex items-center gap-1.5 transition-all cursor-pointer shadow-sm"
            title="Réinitialiser tous les filtres"
          >
            <RotateCcw className="w-3 h-3" />
            <span className="hidden sm:inline">Reset</span>
          </button>
        )}

        {/* Bouton de rafraîchissement de la liste des jeux */}
        {onRefresh && (
          <button
            type="button"
            onClick={onRefresh}
            disabled={isRefreshing}
            className="h-8 px-2 rounded-xl bg-slate-950/70 hover:bg-slate-950/90 border border-slate-800 hover:border-slate-700/80 text-slate-300 hover:text-white text-xs flex items-center justify-center transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
            title="Actualiser la liste des jeux et sauvegardes"
          >
            <RefreshCw
              className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin text-indigo-400" : ""}`}
            />
          </button>
        )}

        {/* Compteur discret des résultats */}
        <div className="text-[11px] text-slate-400 font-mono px-2 hidden md:flex items-center gap-1 shrink-0">
          <span className="text-slate-200 font-semibold">{filteredCount}</span>
          <span className="text-slate-600">/</span>
          <span className="text-slate-400">{totalGames}</span>
        </div>
      </div>
    </div>
  );
}

