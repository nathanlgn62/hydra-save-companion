import AlertBanner from "./components/banner/alert-banner";
import Footer from "./components/footer";
import GameCard from "./components/game-card";
import Header from "./components/header";
import ToastContainer from "./components/toast/toast-container";
import { useGames } from "./hooks/useGames";
import { useSettings } from "./hooks/useSettings";
import { useWatcher } from "./hooks/useWatcher";

export default function App() {
  const { data: games = [], isLoading } = useGames();
  const { updateSettings } = useSettings();

  useWatcher();

  return (
    <div className="w-screen h-screen flex flex-col bg-slate-950 text-slate-100 select-none overflow-hidden font-sans">
      <Header />

      <main className="flex flex-col flex-1 p-4 overflow-y-auto min-h-0">
        <AlertBanner />

        {isLoading ? (
          <div className="flex-1 flex flex-col items-center justify-center gap-3 text-slate-400">
            <div className="w-6 h-6 border-2 border-slate-600 border-t-indigo-500 rounded-full animate-spin" />
            <span className="text-sm">
              Recherche des jeux et sauvegardes...
            </span>
          </div>
        ) : games.length > 0 ? (
          <div className="flex flex-row gap-4 flex-wrap content-start">
            {games.map((game) => (
              <GameCard key={game.objectId} game={game} />
            ))}
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
