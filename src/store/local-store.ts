import type { CurrentGame, LocalStore, Stats } from "../game/types";

const STORAGE_KEY = "hecatordle_store";
const PLAYER_ID_KEY = "hecatordle_player_id";

const EMPTY_STATS: Stats = {
  games_played: 0,
  games_won: 0,
  current_streak: 0,
  max_streak: 0,
  last_puzzle_date: null,
  guess_distribution: {
    "1-20": 0,
    "21-40": 0,
    "41-60": 0,
    "61-80": 0,
    "81-100": 0,
    "101-120": 0,
    "121-134": 0,
  },
};

function safeRandomId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

export function getOrCreatePlayerId(): string {
  const existing = localStorage.getItem(PLAYER_ID_KEY);
  if (existing) {
    return existing;
  }

  const id = safeRandomId();
  localStorage.setItem(PLAYER_ID_KEY, id);
  return id;
}

export function getTodayUTCDateString(now = new Date()): string {
  const year = now.getUTCFullYear();
  const month = String(now.getUTCMonth() + 1).padStart(2, "0");
  const day = String(now.getUTCDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function createDefaultStore(): LocalStore {
  return {
    player_id: getOrCreatePlayerId(),
    current_game: null,
    stats: { ...EMPTY_STATS, guess_distribution: { ...EMPTY_STATS.guess_distribution } },
  };
}

export function loadStore(): LocalStore {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return createDefaultStore();
  }

  try {
    const parsed = JSON.parse(raw) as Partial<LocalStore>;
    const merged: LocalStore = {
      player_id: parsed.player_id ?? getOrCreatePlayerId(),
      current_game: parsed.current_game ?? null,
      stats: {
        ...EMPTY_STATS,
        ...parsed.stats,
        guess_distribution: {
          ...EMPTY_STATS.guess_distribution,
          ...parsed.stats?.guess_distribution,
        },
      },
    };
    return merged;
  } catch {
    return createDefaultStore();
  }
}

export function saveStore(store: LocalStore): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
}

function getGuessBucket(guessCount: number): keyof Stats["guess_distribution"] {
  if (guessCount <= 20) return "1-20";
  if (guessCount <= 40) return "21-40";
  if (guessCount <= 60) return "41-60";
  if (guessCount <= 80) return "61-80";
  if (guessCount <= 100) return "81-100";
  if (guessCount <= 120) return "101-120";
  return "121-134";
}

function isNextUTCDate(prevDate: string, currentDate: string): boolean {
  const prev = new Date(`${prevDate}T00:00:00.000Z`).getTime();
  const current = new Date(`${currentDate}T00:00:00.000Z`).getTime();
  return current - prev === 24 * 60 * 60 * 1000;
}

export function applyGameToStats(
  stats: Stats,
  game: Pick<CurrentGame, "puzzle_date" | "guesses" | "won">,
): Stats {
  const next: Stats = {
    ...stats,
    guess_distribution: { ...stats.guess_distribution },
    games_played: stats.games_played + 1,
    games_won: stats.games_won + (game.won ? 1 : 0),
    last_puzzle_date: game.puzzle_date,
  };

  if (game.won) {
    if (stats.last_puzzle_date && isNextUTCDate(stats.last_puzzle_date, game.puzzle_date)) {
      next.current_streak = stats.current_streak + 1;
    } else {
      next.current_streak = 1;
    }
    next.max_streak = Math.max(stats.max_streak, next.current_streak);
  } else {
    next.current_streak = 0;
  }

  next.guess_distribution[getGuessBucket(game.guesses.length)] += 1;

  return next;
}
