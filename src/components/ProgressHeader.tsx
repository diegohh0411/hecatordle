import { GRID_COUNT, MAX_GUESSES } from "../game/types";

interface ProgressHeaderProps {
  solvedCount: number;
  guessCount: number;
  currentInput: string;
  elapsedSeconds: number;
  isDarkMode: boolean;
  completed: boolean;
  onToggleDarkMode: () => void;
  onOpenStats: () => void;
  onGiveUp: () => void;
}

function formatElapsed(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
}

export function ProgressHeader({
  solvedCount,
  guessCount,
  currentInput,
  elapsedSeconds,
  isDarkMode,
  completed,
  onToggleDarkMode,
  onOpenStats,
  onGiveUp,
}: ProgressHeaderProps) {
  const progress = (solvedCount / GRID_COUNT) * 100;

  return (
    <header className="progress-header">
      <div className="progress-top">
        <h1>Hecatordle</h1>
        <div className="header-actions">
          <button type="button" onClick={onToggleDarkMode} aria-label="Toggle dark mode">
            {isDarkMode ? "☀" : "☾"}
          </button>
          <button type="button" onClick={onOpenStats} aria-label="Open stats">
            📊
          </button>
          <button type="button" onClick={onGiveUp} disabled={completed}>
            Give up
          </button>
        </div>
      </div>
      <div className="progress-meta">
        <span>{solvedCount}/{GRID_COUNT} solved</span>
        <span>{guessCount}/{MAX_GUESSES} guesses</span>
        <span>{formatElapsed(elapsedSeconds)}</span>
      </div>
      <div className="progress-bar" role="progressbar" aria-valuenow={solvedCount} aria-valuemin={0} aria-valuemax={GRID_COUNT}>
        <div style={{ width: `${progress}%` }} />
      </div>
      <div className="input-preview" aria-live="polite">
        {currentInput.padEnd(5, "·")}
      </div>
    </header>
  );
}
