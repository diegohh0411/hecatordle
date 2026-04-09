# Hecatordle

Hecatordle is a 128-word Wordle variant built with React + TypeScript + Vite, with optional Supabase daily puzzle delivery and Tauri desktop packaging.

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```

## Daily puzzle source

- If `VITE_SUPABASE_URL` and `VITE_SUPABASE_ANON_KEY` are set, the app fetches the UTC daily puzzle from Supabase (`daily_puzzles` table).
- If these values are not set, the app falls back to a deterministic local daily puzzle generator for development.

## Tauri

```bash
npm run tauri dev
```
