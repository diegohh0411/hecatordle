import { useMemo } from "react";

export function Celebration({ show }: { show: boolean }) {
  const pieces = useMemo(
    () => Array.from({ length: 60 }, (_, i) => ({
      id: i,
      left: `${Math.random() * 100}%`,
      delay: `${Math.random() * 1.5}s`,
      duration: `${2 + Math.random() * 2}s`,
    })),
    [],
  );

  if (!show) {
    return null;
  }

  return (
    <div className="celebration" aria-hidden="true">
      {pieces.map((piece) => (
        <span
          key={piece.id}
          className="confetti"
          style={{ left: piece.left, animationDelay: piece.delay, animationDuration: piece.duration }}
        />
      ))}
    </div>
  );
}
