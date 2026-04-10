import { LocalStore, PlayerStats, GameState } from '../game/types';

const STORAGE_KEY = 'hecatordle_data';
const PLAYER_ID_KEY = 'hecatordle_player_id';

const DEFAULT_STATS: PlayerStats = {
  gamesPlayed: 0,
  gamesWon: 0,
  currentStreak: 0,
  maxStreak: 0,
  lastPuzzleDate: null,
  guessDistribution: {
    '1-20': 0, '21-40': 0, '41-60': 0, '61-80': 0,
    '81-100': 0, '101-120': 0, '121-134': 0,
  },
};

export function getOrCreatePlayerId(): string {
  let id = localStorage.getItem(PLAYER_ID_KEY);
  if (!id) {
    id = crypto.randomUUID();
    localStorage.setItem(PLAYER_ID_KEY, id);
  }
  return id;
}

export function loadLocalStore(): LocalStore {
  const data = localStorage.getItem(STORAGE_KEY);
  const playerId = getOrCreatePlayerId();
  
  if (!data) {
    return {
      playerId,
      currentGame: null,
      stats: { ...DEFAULT_STATS },
    };
  }

  try {
    const parsed = JSON.parse(data);
    return {
      playerId,
      currentGame: parsed.currentGame || null,
      stats: parsed.stats || { ...DEFAULT_STATS },
    };
  } catch (e) {
    console.error('Failed to parse local storage', e);
    return {
      playerId,
      currentGame: null,
      stats: { ...DEFAULT_STATS },
    };
  }
}

export function saveLocalStore(store: LocalStore) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
}

function computeStats(stats: PlayerStats, game: GameState): PlayerStats {
  const newStats = { ...stats, guessDistribution: { ...stats.guessDistribution } };
  newStats.gamesPlayed += 1;

  if (game.won) {
    newStats.gamesWon += 1;
    newStats.currentStreak += 1;
    if (newStats.currentStreak > newStats.maxStreak) {
      newStats.maxStreak = newStats.currentStreak;
    }
    const count = game.guesses.length;
    if (count <= 20) newStats.guessDistribution['1-20']++;
    else if (count <= 40) newStats.guessDistribution['21-40']++;
    else if (count <= 60) newStats.guessDistribution['41-60']++;
    else if (count <= 80) newStats.guessDistribution['61-80']++;
    else if (count <= 100) newStats.guessDistribution['81-100']++;
    else if (count <= 120) newStats.guessDistribution['101-120']++;
    else newStats.guessDistribution['121-134']++;
  } else {
    newStats.currentStreak = 0;
  }

  newStats.lastPuzzleDate = game.puzzleDate;
  return newStats;
}

/** Call as soon as a game completes (same session). Keeps currentGame intact for display. */
export function recordCompletedGame(store: LocalStore): LocalStore {
  const { currentGame, stats } = store;
  if (!currentGame || !currentGame.completed) return store;
  // Guard: already recorded for this puzzle date
  if (stats.lastPuzzleDate === currentGame.puzzleDate) return store;

  return { ...store, stats: computeStats(stats, currentGame) };
}

/** Call on next-day load. Clears currentGame; skips stat update if already recorded. */
export function archiveCurrentGameToStats(store: LocalStore): LocalStore {
  const { currentGame, stats } = store;
  if (!currentGame) return store;

  const newStats = stats.lastPuzzleDate === currentGame.puzzleDate
    ? stats
    : computeStats(stats, currentGame);

  return { ...store, currentGame: null, stats: newStats };
}

export function initNewGame(words: string[], date: string): GameState {
  return {
    puzzleDate: date,
    guesses: [],
    targetWords: words,
    solved: new Array(words.length).fill(false),
    completed: false,
    won: false,
  };
}
