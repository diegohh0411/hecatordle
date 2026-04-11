import { createClient } from '@supabase/supabase-js';
import { GameState, PlayerStats } from '../game/types';

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL || '';
const supabaseKey = import.meta.env.VITE_SUPABASE_ANON_KEY || '';

export const supabase = createClient(supabaseUrl, supabaseKey);

export async function fetchDailyPuzzle(date: string): Promise<string[] | null> {
  if (!supabaseUrl || !supabaseKey) return null;

  const { data, error } = await supabase
    .from('daily_puzzles')
    .select('words')
    .eq('puzzle_date', date)
    .single();

  if (error || !data) {
    console.error('Error fetching daily puzzle:', error);
    return null;
  }

  return data.words;
}

export async function syncTelemetry(
  playerId: string,
  stats: PlayerStats,
  gameState?: GameState | null
) {
  if (!supabaseUrl || !supabaseKey) return;

  const payload = {
    player_id: playerId,
    games_played: stats.gamesPlayed,
    games_won: stats.gamesWon,
    current_streak: stats.currentStreak,
    max_streak: stats.maxStreak,
    last_puzzle_date: stats.lastPuzzleDate,
    guess_distribution: stats.guessDistribution,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    language: navigator.language,
    user_agent: navigator.userAgent,
    screen_width: window.innerWidth,
    screen_height: window.innerHeight,
    current_game_guesses: gameState?.guesses.length || 0,
    current_game_solved: gameState?.solved.filter(s => s).length || 0,
    last_seen_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };

  const { error } = await supabase
    .from('player_telemetry')
    .upsert(payload, { onConflict: 'player_id' });

  if (error) {
    console.error('Error syncing telemetry:', error);
  }
}
