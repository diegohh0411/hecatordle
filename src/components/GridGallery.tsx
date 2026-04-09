import type { GridEvaluation } from "../game/types";
import { MiniGrid } from "./MiniGrid";

interface GridGalleryProps {
  grids: GridEvaluation[];
}

export function GridGallery({ grids }: GridGalleryProps) {
  return (
    <div className="grid-gallery">
      {grids.map((grid, index) => (
        <MiniGrid key={index} index={index} rows={grid.rows} solved={grid.solved} />
      ))}
    </div>
  );
}
