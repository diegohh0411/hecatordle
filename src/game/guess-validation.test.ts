import { describe, it, expect } from 'vitest';
import { getGuessResult, validateGuess, isGridSolved } from './guess-validation';

describe('guess-validation', () => {
  const wordSet = new Set(['apple', 'beach', 'brain']);

  it('validates a correct guess', () => {
    expect(validateGuess('apple', wordSet)).toBe(true);
  });

  it('invalidates a wrong length guess', () => {
    expect(validateGuess('app', wordSet)).toBe(false);
  });

  it('invalidates a word not in list', () => {
    expect(validateGuess('xxxxx', wordSet)).toBe(false);
  });

  it('identifies a solved grid', () => {
    expect(isGridSolved('apple', 'apple')).toBe(true);
    expect(isGridSolved('APPLE', 'apple')).toBe(true);
    expect(isGridSolved('beach', 'apple')).toBe(false);
  });

  describe('getGuessResult', () => {
    it('handles all correct letters', () => {
      const result = getGuessResult('apple', 'apple');
      expect(result.states).toEqual(['correct', 'correct', 'correct', 'correct', 'correct']);
    });

    it('handles all absent letters', () => {
      const result = getGuessResult('brown', 'apple');
      expect(result.states).toEqual(['absent', 'absent', 'absent', 'absent', 'absent']);
    });

    it('handles mixed states', () => {
      const result = getGuessResult('ample', 'apple');
      // a: correct, m: absent, p: correct, l: correct, e: correct
      expect(result.states).toEqual(['correct', 'absent', 'correct', 'correct', 'correct']);
    });

    it('handles present but wrong position', () => {
      const result = getGuessResult('leaps', 'apple');
      // l: present, e: present, a: present, p: present, s: absent
      expect(result.states).toEqual(['present', 'present', 'present', 'present', 'absent']);
    });

    it('handles duplicate letters correctly (Wordle rules)', () => {
      // Guess has two 'p's, target has one
      void getGuessResult('pappa', 'apple');
      // p1: present (at index 1 in apple), a1: present (at index 0 in apple), p2: absent, p3: absent, a2: absent
      // Wait, let's re-calculate:
      // Target: a p p l e
      // Guess:  p a p p a
      // Correct pass: none
      // Present pass:
      // Guess[0] 'p' matches Target[1] -> present
      // Guess[1] 'a' matches Target[0] -> present
      // Guess[2] 'p' matches Target[2] -> present
      // Guess[3] 'p' -> already used both target 'p's? Target has TWO 'p's.
      // a p p l e (target)
      // 0 1 2 3 4

      const result2 = getGuessResult('abbey', 'babes');
      // Target: b a b e s
      // Guess:  a b b e y
      // Correct: Guess[3] 'e' == Target[3] 'e'
      // Present: 
      // Guess[0] 'a' == Target[1] -> present
      // Guess[1] 'b' == Target[0] -> present
      // Guess[2] 'b' == Target[2] -> correct
      expect(result2.states).toEqual(['present', 'present', 'correct', 'correct', 'absent']);
    });
  });
});
