import { useCallback, useEffect, useMemo, useState } from "react";
import { evaluateGame } from "../game/game-state";
import { isValidGuess, normalizeGuess } from "../game/guess-validation";
import { GRID_COUNT, MAX_GUESSES } from "../game/types";
import type { CurrentGame, LocalStore } from "../game/types";
import { fetchDailyPuzzleWords } from "../services/supabase";
import {
  applyGameToStats,
  getTodayUTCDateString,
  loadStore,
  saveStore,
} from "../store/local-store";

const DARK_MODE_KEY = "hecatordle_dark_mode";

interface UseGameStateResult {
  status: "loading" | "ready" | "error";
  errorMessage: string;
  store: LocalStore | null;
  currentInput: string;
  invalidGuess: boolean;
  solvedCount: number;
  guessesUsed: number;
  completed: boolean;
  won: boolean;
  grids: ReturnType<typeof evaluateGame>["grids"];
  usedLetters: Set<string>;
  elapsedSeconds: number;
  statsOpen: boolean;
  isDarkMode: boolean;
  puzzleSource: "supabase" | "local" | null;
  handleKeyInput: (key: string) => void;
  closeStats: () => void;
  openStats: () => void;
  giveUp: () => void;
  toggleDarkMode: () => void;
  shareResult: () => Promise<void>;
}

function createGame(puzzleDate: string, words: string[]): CurrentGame {
  return {
    puzzle_date: puzzleDate,
    puzzle_words: words,
    guesses: [],
    solved: Array.from({ length: GRID_COUNT }, () => false),
    completed: false,
    won: false,
    stats_recorded: false,
    started_at: Date.now(),
  };
}

function withArchivedGame(store: LocalStore): LocalStore {
  if (!store.current_game || store.current_game.stats_recorded) {
    return store;
  }

  const game = store.current_game;
  const won = game.completed ? game.won : false;

  return {
    ...store,
    stats: applyGameToStats(store.stats, {
      puzzle_date: game.puzzle_date,
      guesses: game.guesses,
      won,
    }),
    current_game: {
      ...game,
      completed: true,
      won,
      stats_recorded: true,
    },
  };
}

export function useGameState(): UseGameStateResult {
  const [status, setStatus] = useState<"loading" | "ready" | "error">("loading");
  const [errorMessage, setErrorMessage] = useState("");
  const [store, setStore] = useState<LocalStore | null>(null);
  const [currentInput, setCurrentInput] = useState("");
  const [invalidGuess, setInvalidGuess] = useState(false);
  const [statsOpen, setStatsOpen] = useState(false);
  const [isDarkMode, setIsDarkMode] = useState(() => localStorage.getItem(DARK_MODE_KEY) === "1");
  const [puzzleSource, setPuzzleSource] = useState<"supabase" | "local" | null>(null);

  useEffect(() => {
    if (isDarkMode) {
      document.body.dataset.theme = "dark";
      localStorage.setItem(DARK_MODE_KEY, "1");
    } else {
      document.body.dataset.theme = "light";
      localStorage.setItem(DARK_MODE_KEY, "0");
    }
  }, [isDarkMode]);

  useEffect(() => {
    let mounted = true;

    async function initialize(): Promise<void> {
      const today = getTodayUTCDateString();
      let nextStore = loadStore();

      if (nextStore.current_game && nextStore.current_game.puzzle_date !== today) {
        nextStore = withArchivedGame(nextStore);
        nextStore = { ...nextStore, current_game: null };
      }

      if (!nextStore.current_game || nextStore.current_game.puzzle_date !== today) {
        const puzzleResult = await fetchDailyPuzzleWords(today);
        if (!mounted) {
          return;
        }

        if (!puzzleResult.ok) {
          setStatus("error");
          setErrorMessage(puzzleResult.error);
          return;
        }

        nextStore = {
          ...nextStore,
          current_game: createGame(today, puzzleResult.words),
        };
        setPuzzleSource(puzzleResult.source);
      }

      saveStore(nextStore);
      if (!mounted) {
        return;
      }

      setStore(nextStore);
      setStatus("ready");
      if (nextStore.current_game.completed) {
        setStatsOpen(true);
      }
    }

    initialize();

    return () => {
      mounted = false;
    };
  }, []);

  const evaluation = useMemo(() => {
    if (!store?.current_game) {
      return evaluateGame([], []);
    }
    return evaluateGame(store.current_game.puzzle_words, store.current_game.guesses);
  }, [store]);

  const usedLetters = useMemo(() => {
    const letters = new Set<string>();
    for (const guess of store?.current_game?.guesses ?? []) {
      for (const letter of guess) {
        letters.add(letter);
      }
    }
    return letters;
  }, [store]);

  const elapsedSeconds = useMemo(() => {
    if (!store?.current_game) {
      return 0;
    }

    const end = store.current_game.completed ? Date.now() : Date.now();
    return Math.max(0, Math.floor((end - store.current_game.started_at) / 1000));
  }, [store]);

  const persistStore = useCallback((updater: (previous: LocalStore) => LocalStore) => {
    setStore((previous) => {
      if (!previous) {
        return previous;
      }
      const next = updater(previous);
      saveStore(next);
      return next;
    });
  }, []);

  const completeGame = useCallback((won: boolean) => {
    persistStore((previous) => {
      if (!previous.current_game || previous.current_game.completed) {
        return previous;
      }

      const game = {
        ...previous.current_game,
        completed: true,
        won,
      };
      const stats = applyGameToStats(previous.stats, {
        puzzle_date: game.puzzle_date,
        guesses: game.guesses,
        won,
      });

      return {
        ...previous,
        stats,
        current_game: {
          ...game,
          stats_recorded: true,
          solved: evaluation.grids.map((grid) => grid.solved),
        },
      };
    });
    setStatsOpen(true);
  }, [evaluation.grids, persistStore]);

  const submitGuess = useCallback(() => {
    if (!store?.current_game || store.current_game.completed) {
      return;
    }

    const guess = normalizeGuess(currentInput);
    if (!isValidGuess(guess)) {
      setInvalidGuess(true);
      setTimeout(() => setInvalidGuess(false), 220);
      return;
    }

    if (store.current_game.guesses.length >= MAX_GUESSES) {
      return;
    }

    const guesses = [...store.current_game.guesses, guess];
    const nextEvaluation = evaluateGame(store.current_game.puzzle_words, guesses);
    const finished = nextEvaluation.allSolved || guesses.length >= MAX_GUESSES;

    persistStore((previous) => {
      if (!previous.current_game) {
        return previous;
      }
      return {
        ...previous,
        current_game: {
          ...previous.current_game,
          guesses,
          solved: nextEvaluation.grids.map((grid) => grid.solved),
          completed: finished,
          won: nextEvaluation.allSolved,
        },
      };
    });

    setCurrentInput("");

    if (finished) {
      setTimeout(() => completeGame(nextEvaluation.allSolved), 0);
    }
  }, [completeGame, currentInput, persistStore, store]);

  const handleKeyInput = useCallback((key: string) => {
    if (!store?.current_game || store.current_game.completed) {
      return;
    }

    if (key === "Enter") {
      submitGuess();
      return;
    }

    if (key === "Backspace") {
      setCurrentInput((value) => value.slice(0, -1));
      return;
    }

    if (/^[a-zA-Z]$/.test(key)) {
      setCurrentInput((value) => {
        if (value.length >= 5) {
          return value;
        }
        return `${value}${key.toLowerCase()}`;
      });
    }
  }, [store, submitGuess]);

  const giveUp = useCallback(() => {
    if (!store?.current_game || store.current_game.completed) {
      return;
    }

    persistStore((previous) => {
      if (!previous.current_game || previous.current_game.completed) {
        return previous;
      }
      return {
        ...previous,
        current_game: {
          ...previous.current_game,
          completed: true,
          won: false,
        },
      };
    });

    completeGame(false);
  }, [completeGame, persistStore, store]);

  const shareResult = useCallback(async () => {
    const game = store?.current_game;
    if (!game) {
      return;
    }

    const solvedCount = evaluation.solvedCount;
    const header = `Hecatordle ${game.puzzle_date}`;
    const line = `${game.won ? "🏆" : "💀"} ${solvedCount}/128 • ${game.guesses.length}/134`;
    const text = `${header}\n${line}`;

    if (navigator.clipboard) {
      await navigator.clipboard.writeText(text);
    }
  }, [evaluation.solvedCount, store]);

  return {
    status,
    errorMessage,
    store,
    currentInput,
    invalidGuess,
    solvedCount: evaluation.solvedCount,
    guessesUsed: store?.current_game?.guesses.length ?? 0,
    completed: store?.current_game?.completed ?? false,
    won: store?.current_game?.won ?? false,
    grids: evaluation.grids,
    usedLetters,
    elapsedSeconds,
    statsOpen,
    isDarkMode,
    puzzleSource,
    handleKeyInput,
    closeStats: () => setStatsOpen(false),
    openStats: () => setStatsOpen(true),
    giveUp,
    toggleDarkMode: () => setIsDarkMode((value) => !value),
    shareResult,
  };
}
