export const GRID_COUNT = 128;
export const WORD_LENGTH = 5;
export const MAX_GUESSES = 134;

export type LetterState = "correct" | "present" | "absent";

export interface TileResult {
  letter: string;
  state: LetterState;
}

export interface GridEvaluation {
  rows: TileResult[][];
  solved: boolean;
  solvedAtGuess: number | null;
}

export interface GameEvaluation {
  grids: GridEvaluation[];
  solvedCount: number;
  allSolved: boolean;
}

export interface Stats {
  games_played: number;
  games_won: number;
  current_streak: number;
  max_streak: number;
  last_puzzle_date: string | null;
  guess_distribution: {
    "1-20": number;
    "21-40": number;
    "41-60": number;
    "61-80": number;
    "81-100": number;
    "101-120": number;
    "121-134": number;
  };
}

export interface CurrentGame {
  puzzle_date: string;
  puzzle_words: string[];
  guesses: string[];
  solved: boolean[];
  completed: boolean;
  won: boolean;
  stats_recorded: boolean;
  started_at: number;
  completed_at: number | null;
}

export interface LocalStore {
  player_id: string;
  current_game: CurrentGame | null;
  stats: Stats;
}
