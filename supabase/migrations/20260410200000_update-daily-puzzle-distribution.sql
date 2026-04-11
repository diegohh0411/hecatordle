-- Update daily puzzle distribution to favor easier words.
-- New distribution: 48 very_easy, 40 easy, 24 normal, 16 hard (Total 128).
-- Expert tier remains excluded.

SELECT cron.unschedule('generate-daily-puzzle');

SELECT cron.schedule('generate-daily-puzzle',
  '0 0 * * *',
  $$
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      d.puzzle_date,
      (SELECT jsonb_agg(word) FROM (
        (SELECT word FROM word_bank WHERE difficulty = 'very_easy' ORDER BY RANDOM() LIMIT 48)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'easy'      ORDER BY RANDOM() LIMIT 40)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'normal'    ORDER BY RANDOM() LIMIT 24)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'hard'      ORDER BY RANDOM() LIMIT 16)
      ) sub)
    FROM (VALUES (CURRENT_DATE), (CURRENT_DATE + 1)) AS d(puzzle_date)
    ON CONFLICT (puzzle_date) DO NOTHING;
  $$
);