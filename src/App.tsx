import { RotateCcw, SearchX } from "lucide-react";
import { useMemo, useState } from "react";
import AlertBanner from "./components/banner/alert-banner";
import Footer from "./components/footer";
import GameCard from "./components/game-card";
import GameFilterBar from "./components/game/game-filter-bar";
import Header from "./components/header";
import ToastContainer from "./components/toast/toast-container";
import { useGames } from "./hooks/useGames";
import { useSettings } from "./hooks/useSettings";
import { useWatcher } from "./hooks/useWatcher";
import { SaveStatusFilter } from "./types";
import { getGameStatusCategory } from "./utils/syncBadge";

export default function App() {
  const {
    data: games = [],
    isLoading,
    refetch,
    isFetching,
  } = useGames();
  const { updateSettings } = useSettings();

  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<SaveStatusFilter>("all");

  useWatcher();

  // Mappe chaque jeu à son statut pour un filtrage efficace
  const gameCategories = useMemo(() => {
    const map = new Map<string, SaveStatusFilter>();
    games.forEach((g) => {
      map.set(
        g.objectId,
        getGameStatusCategory(
          g.ludasaviPathExists,
          g.lastLocalSave,
          g.lastRemoteSave,
        ),
      );
    });
    return map;
  }, [games]);

  // Compteurs par statut
  const statusCounts = useMemo(() => {
    const counts: Record<SaveStatusFilter, number> = {
      all: games.length,
      "up-to-date": 0,
      "local-newer": 0,
      "cloud-newer": 0,
      "to-download": 0,
      "never-synced": 0,
      "no-save": 0,
    };

    games.forEach((g) => {
      const cat = gameCategories.get(g.objectId) || "no-save";
      counts[cat] = (counts[cat] || 0) + 1;
    });

    return counts;
  }, [games, gameCategories]);

  // Filtrage combiné : recherche sur titre + statut
  const filteredGames = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();

    return games.filter((game) => {
      if (query && !game.title.toLowerCase().includes(query)) {
        return false;
      }

      if (statusFilter !== "all") {
        const cat = gameCategories.get(game.objectId);
        if (cat !== statusFilter) {
          return false;
        }
      }

      return true;
    });
  }, [games, searchQuery, statusFilter, gameCategories]);

  const handleResetFilters = () => {
    setSearchQuery("");
    setStatusFilter("all");
  };

  return (
    <div className="w-screen h-screen flex flex-col bg-slate-950 text-slate-100 select-none overflow-hidden font-sans">
      <Header />

      <main className="flex flex-col flex-1 px-4 pt-4 pb-18 overflow-y-auto min-h-0 relative">
        <AlertBanner />

        {isLoading ? (
          <div className="flex-1 flex flex-col items-center justify-center gap-3 text-slate-400">
            <div className="w-6 h-6 border-2 border-slate-600 border-t-indigo-500 rounded-full animate-spin" />
            <span className="text-sm">
              Recherche des jeux et sauvegardes...
            </span>
          </div>
        ) : games.length > 0 ? (
          <div className="flex flex-col flex-1">
            <GameFilterBar
              searchQuery={searchQuery}
              onSearchChange={setSearchQuery}
              statusFilter={statusFilter}
              onStatusFilterChange={setStatusFilter}
              totalGames={games.length}
              filteredCount={filteredGames.length}
              statusCounts={statusCounts}
              onReset={handleResetFilters}
              onRefresh={() => refetch()}
              isRefreshing={isFetching}
            />

            {filteredGames.length > 0 ? (
              <div className="grid grid-cols-2 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-4 items-start w-full">
                {filteredGames.map((game) => (
                  <GameCard key={game.objectId} game={game} />
                ))}
              </div>
            ) : (
              <div className="flex-1 flex flex-col items-center justify-center p-8 text-center max-w-md mx-auto">
                <div className="p-3.5 bg-slate-900 border border-slate-800 rounded-2xl mb-3 text-slate-500 shadow-xl">
                  <SearchX className="w-8 h-8 stroke-[1.5]" />
                </div>
                <h3 className="text-base font-semibold text-slate-200 mb-1">
                  Aucun résultat
                </h3>
                <p className="text-xs text-slate-400 leading-relaxed mb-4">
                  Aucun jeu ne correspond à vos filtres actuels.
                </p>
                <button
                  type="button"
                  onClick={handleResetFilters}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-medium bg-indigo-600 hover:bg-indigo-500 text-white transition cursor-pointer shadow-md shadow-indigo-600/20"
                >
                  <RotateCcw className="w-3.5 h-3.5" />
                  Réinitialiser les filtres
                </button>
              </div>
            )}
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center p-8 text-center max-w-md mx-auto">
            <div className="p-4 bg-slate-900/80 border border-slate-800 rounded-2xl mb-4 text-slate-500 shadow-xl">
              <svg
                className="w-10 h-10 stroke-[1.5]"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M9 10h.01M15 10h.01M9 15s1.5-2 3-2 3 2 3 2"
                />
              </svg>
            </div>
            <h3 className="text-base font-semibold text-slate-200 mb-2">
              Aucun jeu détecté
            </h3>
            <p className="text-sm text-slate-400 leading-relaxed mb-6">
              Vérifiez que <span className="font-bold">Hydra Launcher</span> est
              bien installé et que des jeux y sont configurés ou ajoutés à votre
              bibliothèque.
            </p>
            <button
              type="button"
              onClick={() => updateSettings({ demoMode: true })}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-medium bg-indigo-600 hover:bg-indigo-500 text-white shadow-lg shadow-indigo-500/20 transition cursor-pointer"
            >
              🎮 Activer le Mode Démo / Simulation
            </button>
          </div>
        )}
      </main>

      <ToastContainer />

      <Footer />
    </div>
  );
}
