import React, { useState, useEffect } from 'react';

interface ProgressHeaderProps {
  solvedCount: number;
  totalWords: number;
  guessCount: number;
  maxGuesses: number;
  completed: boolean;
}

export const ProgressHeader: React.FC<ProgressHeaderProps> = ({
  solvedCount,
  totalWords,
  guessCount,
  maxGuesses,
  completed,
}) => {
  const [seconds, setSeconds] = useState(0);

  useEffect(() => {
    if (completed) return;
    
    const interval = setInterval(() => {
      setSeconds(s => s + 1);
    }, 1000);
    
    return () => clearInterval(interval);
  }, [completed]);

  const formatTime = (s: number) => {
    const mins = Math.floor(s / 60);
    const secs = s % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const solvedPercentage = (solvedCount / totalWords) * 100;

  return (
    <header className="progress-header">
      <div className="stats-row">
        <div className="stat">
          <span className="label">Solved</span>
          <span className="value">{solvedCount}/{totalWords}</span>
        </div>
        <div className="stat">
          <span className="label">Guesses</span>
          <span className="value">{guessCount}/{maxGuesses}</span>
        </div>
        <div className="stat">
          <span className="label">Time</span>
          <span className="value">{formatTime(seconds)}</span>
        </div>
      </div>
      <div className="progress-bar-container">
        <div 
          className="progress-bar-fill" 
          style={{ width: `${solvedPercentage}%` }}
        />
      </div>
    </header>
  );
};
