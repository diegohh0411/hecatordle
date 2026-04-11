-- Update daily puzzle generation to 5 tiers (very_easy, easy, normal, hard, expert).
-- We now select exactly 32 words from the top 4 tiers, ignoring 'expert' tier, to make 128 words.
-- Generates both today's and tomorrow's puzzles so users in any timezone
-- always have the correct puzzle ready at midnight local time.

SELECT cron.unschedule('generate-daily-puzzle');

SELECT cron.schedule('generate-daily-puzzle',
  '0 0 * * *',
  $$
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      d.puzzle_date,
      (SELECT jsonb_agg(word) FROM (
        (SELECT word FROM word_bank WHERE difficulty = 'very_easy' ORDER BY RANDOM() LIMIT 32)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'easy'      ORDER BY RANDOM() LIMIT 32)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'normal'    ORDER BY RANDOM() LIMIT 32)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'hard'      ORDER BY RANDOM() LIMIT 32)
      ) sub)
    FROM (VALUES (CURRENT_DATE), (CURRENT_DATE + 1)) AS d(puzzle_date)
    ON CONFLICT (puzzle_date) DO NOTHING;
  $$
);