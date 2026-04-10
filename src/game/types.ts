export type LetterState = 'absent' | 'present' | 'correct' | 'empty';

export interface GuessResult {
  letters: string[];
  states: LetterState[];
}

export interface GameState {
  puzzleDate: string; // 'YYYY-MM-DD'
  guesses: string[];  // Up to 134 guesses
  targetWords: string[]; // 128 words
  solved: boolean[];  // array of 128 booleans
  completed: boolean;
  won: boolean;
}

export interface PlayerStats {
  gamesPlayed: number;
  gamesWon: number;
  currentStreak: number;
  maxStreak: number;
  lastPuzzleDate: string | null;
  guessDistribution: {
    '1-20': number;
    '21-40': number;
    '41-60': number;
    '61-80': number;
    '81-100': number;
    '101-120': number;
    '121-134': number;
  };
}

export interface LocalStore {
  playerId: string;
  currentGame: GameState | null;
  stats: PlayerStats;
}
