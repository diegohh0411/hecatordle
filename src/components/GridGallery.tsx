import React from 'react';
import { MiniGrid } from './MiniGrid';

interface GridGalleryProps {
  targetWords: string[];
  guesses: string[];
  currentGuess: string;
  solved: boolean[];
  isInvalidGuess: boolean;
}

export const GridGallery: React.FC<GridGalleryProps> = ({
  targetWords,
  guesses,
  currentGuess,
  solved,
  isInvalidGuess,
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
          isInvalidGuess={isInvalidGuess}
        />
      ))}
    </div>
  );
};
