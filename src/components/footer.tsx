import { getRunningGame } from "../stores/gameStore";

export default function Footer() {
  const runningGame = getRunningGame();

  return (
    <footer className="h-8 bg-slate-950 border-t border-slate-800/80 px-3 flex items-center justify-between text-[11px] text-slate-500 shrink-0 select-none">
      <div className="flex items-center gap-2">
        {runningGame ? (
          <>
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="text-slate-300 font-medium">
              Jeu en cours :{" "}
              <span className="text-emerald-400">{runningGame}</span>
            </span>
          </>
        ) : (
          <>
            <span className="w-2 h-2 rounded-full bg-slate-600" />
            <span>En attente d'un jeu</span>
          </>
        )}
      </div>

      <div>
        <span>Google Drive : 4.2 / 15 Go</span>
      </div>
    </footer>
  );
}
