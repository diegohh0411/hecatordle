import React, { useMemo } from 'react';
import { getGuessResult, isGridSolved } from '../game/guess-validation';
import { LetterState } from '../game/types';

interface MiniGridProps {
  targetWord: string;
  guesses: string[];
  currentGuess: string;
  isSolved: boolean;
  index: number;
}

export const MiniGrid: React.FC<MiniGridProps> = ({
  targetWord,
  guesses,
  currentGuess,
  isSolved,
  index,
}) => {
  const rows = useMemo(() => {
    // If solved, freeze at the guess that solved it
    let visibleGuesses = guesses;
    if (isSolved) {
      const solveIndex = guesses.findIndex(g => isGridSolved(g, targetWord));
      visibleGuesses = solveIndex !== -1 ? guesses.slice(0, solveIndex + 1) : guesses;
    }

    const res = visibleGuesses.map(g => getGuessResult(g, targetWord));

    // If not solved and there is space, show current guess preview
    if (!isSolved && guesses.length < 134) {
      const letters = currentGuess.padEnd(5, ' ').split('');
      const states: LetterState[] = new Array(5).fill('empty');
      res.push({ letters, states });
    }

    return res;
  }, [targetWord, guesses, currentGuess, isSolved]);

  return (
    <div className={`mini-grid ${isSolved ? 'solved' : ''}`}>
      <div className="mini-grid-header">
        <span className="grid-index">#{index + 1}</span>
        {isSolved && <span className="solved-label">SOLVED</span>}
      </div>
      <div className="mini-grid-rows">
        {rows.map((row, i) => (
          <div key={i} className="row">
            {row.letters.map((letter, j) => (
              <div key={j} className={`tile ${row.states[j]}`}>
                {letter.trim()}
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
};
