import React, { useMemo, useRef, useEffect } from 'react';
import { getGuessResult } from '../game/guess-validation';
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
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom of guesses
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [guesses.length]);

  const rows = useMemo(() => {
    const res = guesses.map(g => getGuessResult(g, targetWord));
    
    // If not solved and there is space, show current guess
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
      <div className="mini-grid-rows" ref={scrollRef}>
        {rows.map((row, i) => (
          <div key={i} className="row">
            {row.letters.map((letter, j) => (
              <div key={j} className={`tile ${row.states[j]}`}>
                {letter.trim()}
              </div>
            ))}
          </div>
        ))}
        {/* Placeholder for remaining rows to maintain consistent height if needed */}
        {/* Actually, let's keep it scrollable and just show used + 1 */}
      </div>
    </div>
  );
};
