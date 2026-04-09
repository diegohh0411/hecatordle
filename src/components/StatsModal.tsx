import type { Stats } from "../game/types";

interface StatsModalProps {
  open: boolean;
  stats: Stats;
  lastResult: { won: boolean; guesses: number; solved: number } | null;
  onClose: () => void;
  onShare: () => void;
}

export function StatsModal({ open, stats, lastResult, onClose, onShare }: StatsModalProps) {
  if (!open) {
    return null;
  }

  const winRate = stats.games_played === 0 ? 0 : Math.round((stats.games_won / stats.games_played) * 100);
  const maxCount = Math.max(1, ...Object.values(stats.guess_distribution));

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(event) => event.stopPropagation()}>
        <h2>Stats</h2>
        <div className="stat-grid">
          <div><strong>{stats.games_played}</strong><span>Played</span></div>
          <div><strong>{winRate}%</strong><span>Win %</span></div>
          <div><strong>{stats.current_streak}</strong><span>Current streak</span></div>
          <div><strong>{stats.max_streak}</strong><span>Max streak</span></div>
        </div>

        <h3>Guess distribution</h3>
        <div className="distribution">
          {Object.entries(stats.guess_distribution).map(([bucket, count]) => (
            <div key={bucket} className="distribution-row">
              <span>{bucket}</span>
              <div className="distribution-bar" style={{ width: `${(count / maxCount) * 100}%` }}>{count}</div>
            </div>
          ))}
        </div>

        {lastResult && (
          <p className="last-result">
            Last game: {lastResult.won ? "Win" : "Loss"} • {lastResult.solved}/128 solved • {lastResult.guesses} guesses
          </p>
        )}

        <div className="modal-actions">
          <button type="button" onClick={onShare}>Share</button>
          <button type="button" onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  );
}
