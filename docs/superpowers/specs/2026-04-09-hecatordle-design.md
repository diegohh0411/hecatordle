# Hecatordle — Design Specification

**Date:** 2026-04-09  
**Author:** Diegohh0411 + Qwen Code  
**Status:** Draft

---

## Overview

Hecatordle is a 128-word Wordle variant — the same game mechanics as Sedecordle, but scaled from 16 to 128 words. Players have 134 total guesses; each guess applies to all 128 words simultaneously. The app is deployable as a website on Vercel and compilable as a native desktop app via Tauri 2.0.

### Core Principles

- **Daily puzzle only** — one puzzle per day, everyone solves the same words
- **Client-first** — all game logic and state in the browser; Supabase is read-only puzzle delivery
- **Single codebase** — identical build for Vercel and Tauri
- **Minimal celebration** — confetti, stats, share button

---

## Architecture

### Project Structure

```
hecatordle/
├── src/                          # React frontend (single codebase)
│   ├── game/                     # Pure game logic (no platform coupling)
│   │   ├── word-list.ts          # Bundled dictionary (exported from word_bank, see Word Frequency Tool)
│   │   ├── guess-validation.ts   # Local validation (dictionary check)
│   │   ├── game-state.ts         # 128-grid state management
│   │   └── types.ts              # Core types
│   ├── components/               # React components
│   │   ├── GridGallery.tsx       # 128 MiniGrids in scrollable layout
│   │   ├── MiniGrid.tsx          # Single 5×134 grid column
│   │   ├── Keyboard.tsx          # Global QWERTY keyboard
│   │   ├── ProgressHeader.tsx    # Top bar with counter and progress
│   │   ├── StatsModal.tsx        # Stats overlay
│   │   └── Celebration.tsx       # Completion confetti
│   ├── services/                 # External integrations
│   │   └── supabase.ts           # Supabase client — read-only (fetch daily puzzle)
│   ├── hooks/                    # React hooks
│   │   ├── useGameState.ts       # Game state management
│   │   └── useKeyboard.ts        # Keyboard input handling
│   ├── store/                    # Local state (localStorage — source of truth for game + stats)
│   │   └── local-store.ts        # Player identity, current game, and stats
│   └── App.tsx                   # Root component
├── supabase/                     # Supabase project files
│   ├── migrations/               # SQL migrations
│   │   └── 001_initial.sql       # Tables: word_bank, daily_puzzles
│   └── config.toml               # Supabase CLI config
├── tools/
│   └── word-frequency/           # Rust tool for word frequency calculation
│       ├── Cargo.toml
│       └── src/                  # Book parsing + frequency calculation
├── src-tauri/                    # Tauri desktop wrapper (minimal)
├── docs/superpowers/specs/       # Design docs
└── vite.config.ts                # Build config (Vercel + Tauri)
```

### Build Flow

```
React/TypeScript → Vite build → dist/ → Vercel serves it
                                  → Tauri wraps it
```

Same build output, two deployment targets. No platform-specific code paths.

---

## Player Identity & Local Store

No login. Players are identified by a UUID generated on first visit and persisted in localStorage.

```typescript
// On app init — local-store.ts
function getOrCreatePlayerId(): string {
  const KEY = 'hecatordle_player_id';
  let id = localStorage.getItem(KEY);
  if (!id) {
    id = crypto.randomUUID();
    localStorage.setItem(KEY, id);
  }
  return id;
}
```

### localStorage Schema

`local-store.ts` owns the following shape. All game state and stats live here — Supabase is read-only (puzzle fetch only).

```typescript
interface LocalStore {
  player_id: string;             // UUID, generated once on first visit

  current_game: {
    puzzle_date: string;         // 'YYYY-MM-DD' — detects day rollover
    guesses: string[];           // ordered list of all guesses made (max 134)
    solved: boolean[];           // [128] — true when a grid's word is found
    completed: boolean;          // true when game is over (win or loss)
    won: boolean;
  } | null;

  stats: {
    games_played: number;
    games_won: number;
    current_streak: number;
    max_streak: number;
    last_puzzle_date: string | null;
    guess_distribution: {
      '1-20': number; '21-40': number; '41-60': number;
      '61-80': number; '81-100': number; '101-120': number; '121-134': number;
    };
  };
}
```

On app start, if `current_game.puzzle_date` differs from today, the old game is archived to stats and `current_game` is reset.

---

## Game Mechanics

### Rules

- 128 hidden words (5 letters each)
- Each guess applies to all 128 grids simultaneously
- Each MiniGrid has 134 rows — this is the global guess budget
- **Win:** all 128 words solved before running out of rows
- **Loss:** all 134 rows used without solving all 128 words
- Once a grid is solved, it stops receiving new guesses — subsequent guesses only apply to unsolved grids
- The strategic challenge: distribute guesses efficiently across words you're solving
- Player may give up at any time (counts as a loss)

### Scoring Per Row

- **Green** — correct letter, correct position
- **Yellow** — correct letter, wrong position
- **Gray** — letter not in the word

### Keyboard State

- **Binary**: keys are either "unused" (default appearance) or "used" (dimmed/opaque)
- No color aggregation — a key goes dimmed if it has been typed at least once
- Physical keyboard mirrors on-screen keyboard state

### Input Flow

```
1. User types a 5-letter word
2. LOCAL VALIDATION (<10ms):
   └─ Is it a valid 5-letter word in the dictionary?
   └─ If NO: shake animation, red flash, row not added
   └─ If YES: tiles flip colors immediately (optimistic feedback)

3. STATE UPDATE:
   └─ Each UNSOLVED grid evaluates the guess against its own word
   └─ MiniGrid renders the new row with per-letter results (solved grids unchanged)
   └─ Keyboard marks typed letters as "used"
   └─ Completion counter updates (e.g., "47/128 solved")
   └─ Game state persisted to localStorage
```

### Daily Puzzle Rotation

- Puzzle rotates at midnight UTC
- If a player is mid-game when the puzzle rotates, their session continues with the old puzzle
- New visits get the new day's puzzle

---

## Supabase Schema

### Tables

#### `word_bank` — Universe of valid 5-letter words

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `word` | VARCHAR(5) | PRIMARY KEY | The word |
| `frequency_per_1k` | FLOAT | NOT NULL | Occurrences per 1,000 words in corpus |
| `difficulty` | VARCHAR(10) | DEFAULT 'normal' | Derived: 'easy' >1.0, 'normal' 0.1-1.0, 'hard' <0.1 |
| `sources` | JSONB | | `[{"book": "Frankenstein", "count": 12}]` |
| `created_at` | TIMESTAMPTZ | DEFAULT NOW() | When added |

#### `daily_puzzles` — Daily 128-word puzzle sets

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `puzzle_date` | DATE | PRIMARY KEY | The date for this puzzle |
| `words` | JSONB | NOT NULL | Array of 128 words |
| `generated_at` | TIMESTAMPTZ | DEFAULT NOW() | When generated |

### pg_cron Daily Generation

Difficulty mix: **40% easy (51 words), 40% normal (51 words), 20% hard (26 words)**.

```sql
SELECT cron.schedule('generate-daily-puzzle',
  '0 0 * * *',
  $$
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      CURRENT_DATE,
      (SELECT jsonb_agg(word) FROM (
        SELECT word FROM word_bank WHERE difficulty = 'easy'   ORDER BY RANDOM() LIMIT 51
        UNION ALL
        SELECT word FROM word_bank WHERE difficulty = 'normal' ORDER BY RANDOM() LIMIT 51
        UNION ALL
        SELECT word FROM word_bank WHERE difficulty = 'hard'   ORDER BY RANDOM() LIMIT 26
      ) sub)
    ON CONFLICT (puzzle_date) DO NOTHING;
  $$
);
```

---

## UI Components

### `GridGallery`

- Responsive CSS Grid: **2 columns on mobile → 8 columns on desktop**
- Renders 128 `MiniGrid` components
- Solved grids get a gold border, active grids standard borders
- Each grid labeled with index number (1-128)
- Container scrolls vertically

### `MiniGrid`

- **5 columns × 134 rows** (global guess budget)
- Tall, internally scrollable component
- Each row = one guess, evaluated against this grid's specific word
- Green/yellow/gray tile colors with smooth transitions
- Gold border when solved; solved grids accept no further input and show no new rows

### `Keyboard`

- Standard QWERTY layout
- **Submit (Enter) button on the left, Backspace on the right** — follows Wordle/Sedecordle convention (inverted from most keyboards)
- Backspace removes only the last letter typed (does not clear the row)
- Binary key state: default (unused) or dimmed (used)
- All keypresses broadcast to all 128 grids simultaneously

### `ProgressHeader`

- Fixed top bar
- Counter: "X/128 solved" with progress bar
- Time elapsed
- Settings and stats icons

### `StatsModal`

- Games played, win %, current streak, max streak
- Guess distribution graph (total guesses used, bucketed: 1-20, 21-40, 41-60, 61-80, 81-100, 101-120, 121-134)
- Share button (copyable text summary)

### `Celebration`

- Confetti animation on 128/128 completion
- Stats auto-popup
- Simple and clean

### Color Scheme

- Classic Wordle green (#6aaa64) / yellow (#c9b458) / gray (#787c7e)
- Dark mode toggle
- Gold border (#f5c542) for completed grids
- Minimal, functional, grid-focused aesthetic

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Network failure (puzzle fetch) | Show loading state; retry on reconnect. Game cannot start until puzzle is fetched |
| Midnight UTC rotation | Mid-game session continues with old puzzle (stored in localStorage); new visits get new puzzle |
| Offline mid-game | All game state is in localStorage — play continues uninterrupted; no re-sync needed |
| Word list not seeded | Show "Daily puzzle not available yet" state |

---

## Word Frequency Tool (`tools/word-frequency/`)

A Rust tool maintained in the repo to calculate word frequencies from book corpora.

### Workflow

1. Download free books (OnlineArchive or similar sources)
2. Use tesseract OCR to extract text
3. Parse extracted text to find all 5-letter words
4. Calculate frequency (occurrences per 1,000 words)
5. Track source book/file for each word
6. Output CSV/JSON for import into `word_bank`
7. Export `word_bank` words to `src/game/word-list.ts` — a static JS array bundled with the app, used for local guess validation

### Output Format

```json
{
  "word": "apple",
  "frequency_per_1k": 2.3,
  "difficulty": "easy",
  "sources": [
    {"book": "Frankenstein", "count": 12},
    {"book": "Dracula", "count": 8}
  ]
}
```

### Difficulty Derivation

- `easy`: frequency > 1.0 per 1k
- `normal`: frequency 0.1–1.0 per 1k
- `hard`: frequency < 0.1 per 1k

---

## Non-Goals (Out of Scope)

- Unlimited/random puzzle mode
- Custom puzzle creation
- Mobile app (Tauri mobile is not a target — desktop only)
- Leaderboards or social features beyond shareable results
- Progressive difficulty or themed puzzles
- Multi-language support
