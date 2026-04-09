import { EASY_WORDS, HARD_WORDS, NORMAL_WORDS } from "../game/word-list";

const EASY_COUNT = 51;
const NORMAL_COUNT = 51;
const HARD_COUNT = 26;

type PuzzleFetchResult =
  | { ok: true; words: string[]; source: "supabase" | "local" }
  | { ok: false; error: string };

function hashDate(date: string): number {
  let hash = 0;
  for (let i = 0; i < date.length; i += 1) {
    hash = (hash * 31 + date.charCodeAt(i)) >>> 0;
  }
  return hash;
}

function mulberry32(seed: number): () => number {
  return () => {
    let t = (seed += 0x6d2b79f5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function sampleWithoutReplacement(words: string[], count: number, random: () => number): string[] {
  const pool = [...words];
  for (let i = pool.length - 1; i > 0; i -= 1) {
    const j = Math.floor(random() * (i + 1));
    [pool[i], pool[j]] = [pool[j], pool[i]];
  }
  return pool.slice(0, count);
}

function generateLocalPuzzle(date: string): string[] {
  const random = mulberry32(hashDate(date));
  return [
    ...sampleWithoutReplacement(EASY_WORDS, EASY_COUNT, random),
    ...sampleWithoutReplacement(NORMAL_WORDS, NORMAL_COUNT, random),
    ...sampleWithoutReplacement(HARD_WORDS, HARD_COUNT, random),
  ];
}

function isValidPuzzleWords(words: unknown): words is string[] {
  return (
    Array.isArray(words) &&
    words.length === 128 &&
    words.every((word) => typeof word === "string" && /^[a-z]{5}$/.test(word))
  );
}

export async function fetchDailyPuzzleWords(puzzleDate: string): Promise<PuzzleFetchResult> {
  const supabaseUrl = import.meta.env.VITE_SUPABASE_URL as string | undefined;
  const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY as string | undefined;

  if (!supabaseUrl || !supabaseAnonKey) {
    return { ok: true, words: generateLocalPuzzle(puzzleDate), source: "local" };
  }

  const url = `${supabaseUrl}/rest/v1/daily_puzzles?puzzle_date=eq.${puzzleDate}&select=words`;

  try {
    const response = await fetch(url, {
      headers: {
        apikey: supabaseAnonKey,
        authorization: `Bearer ${supabaseAnonKey}`,
        accept: "application/json",
      },
    });

    if (!response.ok) {
      return { ok: false, error: `Failed to fetch daily puzzle (${response.status})` };
    }

    const rows = (await response.json()) as Array<{ words: unknown }>;
    const words = rows[0]?.words;

    if (!isValidPuzzleWords(words)) {
      return { ok: false, error: "Daily puzzle not available yet" };
    }

    return { ok: true, words, source: "supabase" };
  } catch {
    return { ok: false, error: "Network error while fetching daily puzzle" };
  }
}
