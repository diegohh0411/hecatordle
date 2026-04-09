import { MAX_GUESSES } from "../game/types";
import type { TileResult } from "../game/types";

interface MiniGridProps {
  index: number;
  rows: TileResult[][];
  solved: boolean;
}

export function MiniGrid({ index, rows, solved }: MiniGridProps) {
  return (
    <section className={`mini-grid ${solved ? "solved" : ""}`}>
      <header className="mini-grid-title">#{index + 1}</header>
      <div className="mini-grid-body">
        {Array.from({ length: MAX_GUESSES }, (_, rowIndex) => {
          const row = rows[rowIndex];
          return (
            <div key={rowIndex} className="mini-row">
              {Array.from({ length: 5 }, (_, colIndex) => {
                const tile = row?.[colIndex];
                return (
                  <div key={colIndex} className={`tile ${tile ? `tile-${tile.state}` : "tile-empty"}`}>
                    {tile?.letter ?? ""}
                  </div>
                );
              })}
            </div>
          );
        })}
      </div>
    </section>
  );
}
