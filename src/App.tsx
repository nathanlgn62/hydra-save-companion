import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import Footer from "./components/footer";
import GameCard from "./components/game-card";
import Header from "./components/header";
import { HydraGame } from "./types/game";

interface SaveInfoResponse {
  pathExists: boolean;
  resolvedPath: string | null;
  lastModified: string | null;
}

export default function App() {
  const [loading, setLoading] = useState(true);
  const [games, setGames] = useState<HydraGame[]>([]);
  const [search] = useState<string>("");

  useEffect(() => {
    async function fetchGamesAndSaves() {
      console.log("Fetch Hydra Launcher Installed Games");
      try {
        setLoading(true);
        const installedGames = await invoke<HydraGame[]>("get_installed_games");

        const storedToken = localStorage.getItem("gdrive_token");
        let accessToken = null;
        if (storedToken) {
          accessToken = storedToken.split("|")[0];
        }

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

              let remoteDate = "Jamais";

              if (saveInfo.pathExists && saveInfo.resolvedPath && accessToken) {
                try {
                  const syncStatus = await invoke<any>(
                    "check_game_sync_status",
                    {
                      token: accessToken,
                      gameTitle: game.title,
                      savePath: saveInfo.resolvedPath,
                    },
                  );

                  if (syncStatus.status === "UpToDate") {
                    remoteDate = syncStatus.localTime;
                  } else if (
                    syncStatus.status === "LocalNewer" ||
                    syncStatus.status === "CloudNewer"
                  ) {
                    remoteDate = syncStatus.cloudTime;
                  }
                } catch (e) {
                  remoteDate = "Jamais";
                }
              }

              return {
                ...game,
                lastLocalSave: saveInfo.lastModified ?? "Jamais",
                lastRemoteSave: remoteDate,
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

        const monitoredPayload = gamesWithSaves.map((game) => {
          const rawPath = game.executablePath || game.title;
          const fileName = rawPath.split(/[/\\]/).pop() || rawPath;

          return {
            title: game.title,
            executable_name: fileName,
            save_path: game.savePath ?? null,
          };
        });

        await invoke("set_monitored_games", { games: monitoredPayload });
      } catch (err) {
        console.error("Erreur lors du chargement des jeux :", err);
      } finally {
        setLoading(false);
      }
    }

    fetchGamesAndSaves();
  }, []);

  const filteredGames = games.filter((g) =>
    g.title.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="w-screen h-screen flex flex-col bg-slate-950 text-slate-100 select-none overflow-hidden font-sans">
      <Header />

      <main className="flex flex-col flex-1 p-4 overflow-y-auto min-h-0">
        {/* Banner Info */}
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
            Seuls les jeux installés ou ajoutés via Hydra Launcher sont
            affichés. Les jeux importés des boutiques officielles disposent déjà
            de leur propre sauvegarde cloud.
          </div>
        </div>

        {/* Dynamic Content */}
        {loading ? (
          <div className="flex-1 flex flex-col items-center justify-center gap-3 text-slate-400">
            <div className="w-6 h-6 border-2 border-slate-600 border-t-indigo-500 rounded-full animate-spin" />
            <span className="text-sm">Recherche des jeux et sauvegardes...</span>
          </div>
        ) : filteredGames.length > 0 ? (
          <div className="flex flex-row gap-4 flex-wrap content-start">
            {filteredGames.map((game) => (
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
            <p className="text-sm text-slate-400 leading-relaxed">
              Vérifiez que <span className="font-bold">Hydra Launcher</span> est bien installé et que des jeux y sont configurés ou ajoutés à votre bibliothèque.
            </p>
          </div>
        )}
      </main>

      <Footer />
    </div>
  );
}