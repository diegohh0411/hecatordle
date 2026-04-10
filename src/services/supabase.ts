import { createClient } from '@supabase/supabase-js';

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
