import React from 'react';
import { MiniGrid } from './MiniGrid';

interface GridGalleryProps {
  targetWords: string[];
  guesses: string[];
  currentGuess: string;
  solved: boolean[];
}

export const GridGallery: React.FC<GridGalleryProps> = ({
  targetWords,
  guesses,
  currentGuess,
  solved,
}) => {
  return (
    <div className="grid-gallery">
      {targetWords.map((word, idx) => (
        <MiniGrid
          key={idx}
          index={idx}
          targetWord={word}
          guesses={guesses}
          currentGuess={currentGuess}
          isSolved={solved[idx]}
        />
      ))}
    </div>
  );
};
