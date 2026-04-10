import { useState, useEffect, useCallback } from 'react';
import { GameState, LocalStore } from '../game/types';
import { loadLocalStore, saveLocalStore, archiveCurrentGameToStats, initNewGame } from '../store/local-store';
import { validateGuess, isGridSolved } from '../game/guess-validation';
import { WORD_BANK } from '../game/word-list';
import { fetchDailyPuzzle } from '../services/supabase';

const MAX_GUESSES = 134;
const WORD_COUNT = 128;

/**
 * Returns YYYY-MM-DD in UTC
 */
function getTodayDateString(): string {
  const d = new Date();
  return d.toISOString().split('T')[0];
}

/**
 * Mock function to get 128 words for a date.
 * Used as fallback if Supabase is not available.
 */
function getMockDailyWords(date: string): string[] {
  const words: string[] = [];
  const bankLength = WORD_BANK.length;
  let seed = date.split('-').reduce((acc, part) => acc + parseInt(part), 0);
  
  for (let i = 0; i < WORD_COUNT; i++) {
    seed = (seed * 1664525 + 1013904223) % 4294967296;
    words.push(WORD_BANK[seed % bankLength]);
  }
  return words;
}

export function useGameState() {
  const [store, setStore] = useState<LocalStore>(() => loadLocalStore());
  const [currentGuess, setCurrentGuess] = useState("");
  const [error, setError] = useState<string | null>(null);

  const gameState = store.currentGame;

  // Initialize or update game on mount/date change
  useEffect(() => {
    const init = async () => {
      const today = getTodayDateString();
      let updatedStore = { ...store };

      if (!updatedStore.currentGame || updatedStore.currentGame.puzzleDate !== today) {
        if (updatedStore.currentGame) {
          updatedStore = archiveCurrentGameToStats(updatedStore);
        }
        
        // Try fetching from Supabase first
        let words = await fetchDailyPuzzle(today);
        if (!words) {
          console.warn('Daily puzzle not found in Supabase, falling back to mock.');
          words = getMockDailyWords(today);
        }
        
        updatedStore.currentGame = initNewGame(words, today);
        setStore(updatedStore);
        saveLocalStore(updatedStore);
      }
    };
    
    init();
  }, []);

  const submitGuess = useCallback((guess: string) => {
    if (!gameState || gameState.completed) return;

    if (guess.length !== 5) {
      setError("Too short");
      return;
    }

    if (!validateGuess(guess, WORD_BANK)) {
      setError("Not in word list");
      return;
    }

    const newGuesses = [...gameState.guesses, guess];
    const newSolved = gameState.solved.map((isSolved, idx) => {
      if (isSolved) return true;
      return isGridSolved(guess, gameState.targetWords[idx]);
    });

    const allSolved = newSolved.every(s => s === true);
    const usedAllGuesses = newGuesses.length >= MAX_GUESSES;
    
    const won = allSolved;
    const completed = allSolved || usedAllGuesses;

    const newGameState: GameState = {
      ...gameState,
      guesses: newGuesses,
      solved: newSolved,
      won,
      completed,
    };

    const newStore: LocalStore = {
      ...store,
      currentGame: newGameState,
    };

    setStore(newStore);
    saveLocalStore(newStore);
    setCurrentGuess("");
    setError(null);

    if (completed) {
      // Archive happens on next day load, but we could also archive immediately if we want
      // For now, follow the design: "On app start, if current_game.puzzle_date differs from today, 
      // the old game is archived to stats and current_game is reset."
    }
  }, [gameState, store]);

  const addLetter = useCallback((letter: string) => {
    if (currentGuess.length < 5) {
      setCurrentGuess(prev => prev + letter.toLowerCase());
      setError(null);
    }
  }, [currentGuess]);

  const removeLetter = useCallback(() => {
    setCurrentGuess(prev => prev.slice(0, -1));
    setError(null);
  }, []);

  return {
    gameState,
    stats: store.stats,
    currentGuess,
    error,
    addLetter,
    removeLetter,
    submitGuess: () => submitGuess(currentGuess),
  };
}
