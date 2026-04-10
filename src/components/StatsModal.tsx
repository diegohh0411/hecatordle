import React from 'react';
import { PlayerStats } from '../game/types';

interface StatsModalProps {
  stats: PlayerStats;
  onClose: () => void;
  isOpen: boolean;
}

export const StatsModal: React.FC<StatsModalProps> = ({ stats, onClose, isOpen }) => {
  if (!isOpen) return null;

  const distribution = stats.guessDistribution;
  const maxVal = Math.max(...Object.values(distribution), 1);

  const shareResults = () => {
    const text = `Hecatordle 128-word Wordle
Played: ${stats.gamesPlayed}
Won: ${stats.gamesWon}
Streak: ${stats.currentStreak}
Max Streak: ${stats.maxStreak}`;
    navigator.clipboard.writeText(text);
    alert("Results copied to clipboard!");
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <button className="close-button" onClick={onClose}>×</button>
        <h2>STATISTICS</h2>
        <div className="stats-grid">
          <div className="stat-box">
            <span className="value">{stats.gamesPlayed}</span>
            <span className="label">Played</span>
          </div>
          <div className="stat-box">
            <span className="value">{Math.round((stats.gamesWon / Math.max(stats.gamesPlayed, 1)) * 100)}</span>
            <span className="label">Win %</span>
          </div>
          <div className="stat-box">
            <span className="value">{stats.currentStreak}</span>
            <span className="label">Current Streak</span>
          </div>
          <div className="stat-box">
            <span className="value">{stats.maxStreak}</span>
            <span className="label">Max Streak</span>
          </div>
        </div>

        <h3>GUESS DISTRIBUTION</h3>
        <div className="distribution-chart">
          {Object.entries(distribution).map(([bucket, count]) => (
            <div key={bucket} className="dist-row">
              <div className="dist-label">{bucket}</div>
              <div className="dist-bar-bg">
                <div 
                  className="dist-bar-fill" 
                  style={{ width: `${(count / maxVal) * 100}%` }}
                >
                  {count}
                </div>
              </div>
            </div>
          ))}
        </div>

        <button className="share-button" onClick={shareResults}>SHARE</button>
      </div>
    </div>
  );
};
