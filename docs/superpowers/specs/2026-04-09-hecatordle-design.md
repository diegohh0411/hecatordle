# Hecatordle — Design Specification

**Date:** 2026-04-09  
**Author:** Diegohh0411 + Qwen Code  
**Status:** Draft

---

## Overview

Hecatordle is a 128-word Wordle variant — the same game mechanics as Sedecordle, but scaled from 16 to 128 words. Players get 6 guesses to solve a single word, and each guess applies to all 128 words simultaneously. The app is deployable as a website on Vercel and compilable as a native desktop app via Tauri 2.0.

### Core Principles

- **Daily puzzle only** — one puzzle per day, everyone solves the same words
- **Game integrity** — server-side validation with optimistic client feedback
- **Single codebase** — identical build for Vercel and Tauri
- **Minimal celebration** — confetti, stats, share button

---

## Architecture

### Project Structure

```
hecatordle/
├── src/                          # React frontend (single codebase)
│   ├── game/                     # Pure game logic (no platform coupling)
│   │   ├── word-list.ts          # Daily word selection
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
│   │   ├── supabase.ts           # Supabase client + queries
│   │   └── validation-service.ts # Optimistic UI: local + server validation
│   ├── hooks/                    # React hooks
│   │   ├── useGameState.ts       # Game state management
│   │   └── useKeyboard.ts        # Keyboard input handling
│   ├── store/                    # Local state (localStorage cache)
│   │   └── local-store.ts        # Cache for offline resilience
│   └── App.tsx                   # Root component
├── supabase/                     # Supabase project files
│   ├── migrations/               # SQL migrations
│   │   └── 001_initial.sql       # Tables: word_bank, daily_puzzles, player_guesses, player_stats
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

## Game Mechanics

### Rules

- 128 hidden words (5 letters each)
- Each guess applies to all 128 grids simultaneously
- Each MiniGrid has 128 + 6 rows (128 maximum guesses + 6 buffer)
- No hard guess limit — player keeps guessing until all 128 words are solved
- Game is won when all 128 words are solved
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

3. BACKGROUND VALIDATION (~100-500ms):
   └─ Send guess to Supabase Edge Function
   └─ Server validates: correct word? within time window?
   └─ If server disagrees: row reverts, toast notification
   └─ If server confirms: persist guess, update stats

4. STATE UPDATE:
   └─ Each of the 128 grids evaluates the guess against its own word
   └─ MiniGrid renders the new row with per-letter results
   └─ Keyboard marks typed letters as "used"
   └─ Completion counter updates (e.g., "47/128 solved")
```

### Daily Puzzle Rotation

- Puzzle rotates at midnight UTC
- If a player is mid-game when the puzzle rotates, their session continues with the old puzzle
- New visits get the new day's puzzle
- If pg_cron fails to generate, the edge function generates on first request as fallback

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

#### `player_guesses` — Player guess records

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY DEFAULT gen_random_uuid() | Row ID |
| `player_id` | UUID | NOT NULL | Player identifier |
| `puzzle_date` | DATE | REFERENCES daily_puzzles | Which puzzle |
| `grid_index` | INT | NOT NULL | Which grid (0-127) |
| `guess_number` | INT | NOT NULL | Attempt number (1-6) |
| `word` | VARCHAR(5) | NOT NULL | The guessed word |
| `result` | JSONB | | Per-letter results `[{letter, status}]` |
| `submitted_at` | TIMESTAMPTZ | DEFAULT NOW() | When submitted |

#### `player_stats` — Aggregated player statistics

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `player_id` | UUID | PRIMARY KEY | Player identifier |
| `games_played` | INT | DEFAULT 0 | Total games started |
| `games_won` | INT | DEFAULT 0 | Games with 128/128 solved |
| `current_streak` | INT | DEFAULT 0 | Consecutive daily wins |
| `max_streak` | INT | DEFAULT 0 | Best streak ever |
| `guess_distribution` | JSONB | | Distribution of words solved by guess count: `{"1": 0, "2": 0, "3": 0, "4": 0, "5": 0, "6-10": 0, "11-20": 0, "21+"}` |
| `last_puzzle_date` | DATE | | Last played date |
| `updated_at` | TIMESTAMPTZ | DEFAULT NOW() | Last stats update |

### pg_cron Daily Generation

```sql
SELECT cron.schedule('generate-daily-puzzle',
  '0 0 * * *',
  $$
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      CURRENT_DATE,
      (SELECT jsonb_agg(word) FROM (
        SELECT word FROM word_bank ORDER BY RANDOM() LIMIT 128
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

- **5 columns × 134 rows** (128 guesses + 6 buffer)
- Tall, internally scrollable component
- Each row = one guess, evaluated against this grid's specific word
- Green/yellow/gray tile colors with smooth transitions
- Gold border when solved

### `Keyboard`

- Standard QWERTY layout
- **Submit (Enter) button on the left**
- **Backspace button on the right**
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
- Guess distribution graph (attempts 1-6)
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
| Network failure during guess | Local validation succeeds, server retry in background. Toast if retry fails after 3 attempts |
| pg_cron fails to generate | Edge function generates on first request as fallback |
| Midnight UTC rotation | Mid-game session continues with old puzzle; new visits get new puzzle |
| Offline | localStorage caches day's words and guesses; re-syncs on reconnect |
| Server validation conflict | Row reverts with toast: "Guess was invalid on server" |
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
