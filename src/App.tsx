import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import Footer from "./components/footer";
import GameCard from "./components/game-card";
import Header from "./components/header";
import { HydraGame } from "./types/game";

interface SaveInfoResponse {
  path_exists: boolean;
  resolvedPath: string | null;
  lastModified: string | null;
}

export default function App() {
  const [loading, setLoading] = useState(true);
  const [games, setGames] = useState<HydraGame[]>([]);
  const [search] = useState<string>("");

  useEffect(() => {
    async function fetchGamesAndSaves() {
      try {
        setLoading(true);
        const installedGames = await invoke<HydraGame[]>("get_installed_games");

        const gamesWithSaves = await Promise.all(
          installedGames.map(async (game) => {
            try {
              const saveInfo = await invoke<SaveInfoResponse>(
                "get_game_save_info",
                {
                  appId: game.objectId ?? null,
                  title: game.title,
                },
              );

              console.log(saveInfo);

              return {
                ...game,
                lastLocalSave: saveInfo.lastModified ?? "Jamais",
                savePath: saveInfo.resolvedPath,
              } as HydraGame;
            } catch (err) {
              console.error(`Erreur save info pour ${game.title}:`, err);
              return game;
            }
          }),
        );

        console.log(gamesWithSaves);
        setGames(gamesWithSaves);
      } catch (err) {
        console.error("Erreur lors du chargement des jeux :", err);
      } finally {
        setLoading(false);
      }
    }

    fetchGamesAndSaves();
  }, []);

  return (
    <div className="w-screen h-screen flex flex-col bg-slate-950 text-slate-100 select-none overflow-hidden font-sans">
      <Header />

      <main className="flex flex-row gap-4 flex-wrap p-4 overflow-y-auto flex-1 content-start">
        <div className="flex items-start gap-3 p-3.5 bg-red-500/10 border border-red-500/20 text-red-400 rounded-xl text-sm leading-relaxed backdrop-blur-sm">
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
            Seuls les jeux installés ou ajoutés via Hydra Launcher sont
            affichés. Les jeux importés des boutiques officielles disposent déjà
            de leur propre sauvegarde cloud.
          </div>
        </div>
        {loading ? (
          <div className="flex-1 flex items-center justify-center text-slate-500 text-sm">
            Recherche des jeux et sauvegardes...
          </div>
        ) : (
          games
            .filter((g) => g.title.toLowerCase().includes(search.toLowerCase()))
            .map((game) => <GameCard key={game.objectId} game={game} />)
        )}
      </main>

      <Footer />
    </div>
  );
}
