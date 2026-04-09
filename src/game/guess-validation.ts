import { WORD_SET } from "./word-list";
import { WORD_LENGTH } from "./types";

export function normalizeGuess(value: string): string {
  return value.trim().toLowerCase();
}

export function isValidGuess(value: string): boolean {
  return /^[a-z]+$/.test(value) && value.length === WORD_LENGTH && WORD_SET.has(value);
}
