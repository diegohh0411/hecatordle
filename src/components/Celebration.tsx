import React from 'react';
import Confetti from 'react-confetti';
import { useWindowSize } from 'react-use';

interface CelebrationProps {
  show: boolean;
}

export const Celebration: React.FC<CelebrationProps> = ({ show }) => {
  const { width, height } = useWindowSize();

  if (!show) return null;

  return (
    <div style={{ position: 'fixed', top: 0, left: 0, zIndex: 2000, pointerEvents: 'none' }}>
      <Confetti
        width={width}
        height={height}
        numberOfPieces={200}
        recycle={false}
      />
    </div>
  );
};
