import { LetterState, GuessResult } from './types';

export function validateGuess(guess: string, wordList: string[]): boolean {
  if (guess.length !== 5) return false;
  return wordList.includes(guess.toLowerCase());
}

export function getGuessResult(guess: string, target: string): GuessResult {
  const result: GuessResult = {
    letters: guess.split(''),
    states: new Array(5).fill('absent')
  };

  const targetLetters = target.split('');
  const guessLetters = guess.split('');

  // First pass: find correct letters
  const used = new Array(5).fill(false);
  const guessUsed = new Array(5).fill(false);

  for (let i = 0; i < 5; i++) {
    if (guessLetters[i] === targetLetters[i]) {
      result.states[i] = 'correct';
      used[i] = true;
      guessUsed[i] = true;
    }
  }

  // Second pass: find present but wrong position letters
  for (let i = 0; i < 5; i++) {
    if (guessUsed[i]) continue;
    
    for (let j = 0; j < 5; j++) {
      if (!used[j] && guessLetters[i] === targetLetters[j]) {
        result.states[i] = 'present';
        used[j] = true;
        break;
      }
    }
  }

  return result;
}

export function isGridSolved(guess: string, target: string): boolean {
  return guess.toLowerCase() === target.toLowerCase();
}

/**
 * Returns a map of all letters that have been used at least once in any guess.
 * Design says: "No color aggregation — a key goes dimmed if it has been typed at least once"
 */
export function getKeyboardUsedLetters(guesses: string[]): Set<string> {
  const used = new Set<string>();
  guesses.forEach(guess => {
    guess.split('').forEach(char => used.add(char.toLowerCase()));
  });
  return used;
}
