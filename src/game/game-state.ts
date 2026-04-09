import type { GameEvaluation, GridEvaluation, TileResult } from "./types";

function evaluateGuess(guess: string, solution: string): TileResult[] {
  const result: TileResult[] = guess.split("").map((letter) => ({ letter, state: "absent" }));
  const remaining = new Map<string, number>();

  for (let i = 0; i < solution.length; i += 1) {
    if (guess[i] === solution[i]) {
      result[i].state = "correct";
    } else {
      const letter = solution[i];
      remaining.set(letter, (remaining.get(letter) ?? 0) + 1);
    }
  }

  for (let i = 0; i < guess.length; i += 1) {
    if (result[i].state === "correct") {
      continue;
    }
    const letter = guess[i];
    const count = remaining.get(letter) ?? 0;
    if (count > 0) {
      result[i].state = "present";
      remaining.set(letter, count - 1);
    }
  }

  return result;
}

export function evaluateGame(words: string[], guesses: string[]): GameEvaluation {
  const grids: GridEvaluation[] = words.map(() => ({ rows: [], solved: false, solvedAtGuess: null }));

  for (let guessIndex = 0; guessIndex < guesses.length; guessIndex += 1) {
    const guess = guesses[guessIndex];
    for (let gridIndex = 0; gridIndex < words.length; gridIndex += 1) {
      const grid = grids[gridIndex];
      if (grid.solved) {
        continue;
      }

      const solution = words[gridIndex];
      const row = evaluateGuess(guess, solution);
      grid.rows.push(row);

      if (guess === solution) {
        grid.solved = true;
        grid.solvedAtGuess = guessIndex;
      }
    }
  }

  const solvedCount = grids.reduce((sum, grid) => sum + (grid.solved ? 1 : 0), 0);

  return {
    grids,
    solvedCount,
    allSolved: solvedCount === words.length,
  };
}
