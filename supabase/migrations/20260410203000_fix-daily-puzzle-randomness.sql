-- Fix daily puzzle generation randomness.
-- Using separate INSERT statements for today and tomorrow to guarantee 
-- independent RANDOM() word selection for each day.
-- Maintains distribution: 48 very_easy, 40 easy, 24 normal, 16 hard (Total 128).

SELECT cron.unschedule('generate-daily-puzzle');

SELECT cron.schedule('generate-daily-puzzle',
  '0 0 * * *',
  $$
    -- Generate for today
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      CURRENT_DATE,
      (SELECT jsonb_agg(word) FROM (
        (SELECT word FROM word_bank WHERE difficulty = 'very_easy' ORDER BY RANDOM() LIMIT 48)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'easy'      ORDER BY RANDOM() LIMIT 40)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'normal'    ORDER BY RANDOM() LIMIT 24)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'hard'      ORDER BY RANDOM() LIMIT 16)
      ) sub)
    ON CONFLICT (puzzle_date) DO NOTHING;

    -- Generate for tomorrow
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      CURRENT_DATE + 1,
      (SELECT jsonb_agg(word) FROM (
        (SELECT word FROM word_bank WHERE difficulty = 'very_easy' ORDER BY RANDOM() LIMIT 48)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'easy'      ORDER BY RANDOM() LIMIT 40)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'normal'    ORDER BY RANDOM() LIMIT 24)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'hard'      ORDER BY RANDOM() LIMIT 16)
      ) sub)
    ON CONFLICT (puzzle_date) DO NOTHING;
  $$
);