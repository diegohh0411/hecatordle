-- Update daily puzzle generation to 60% easy / 30% normal / 10% hard
SELECT cron.unschedule('generate-daily-puzzle');

SELECT cron.schedule('generate-daily-puzzle',
  '0 0 * * *',
  $$
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      CURRENT_DATE,
      (SELECT jsonb_agg(word) FROM (
        (SELECT word FROM word_bank WHERE difficulty = 'easy'   ORDER BY RANDOM() LIMIT 77)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'normal' ORDER BY RANDOM() LIMIT 38)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'hard'   ORDER BY RANDOM() LIMIT 13)
      ) sub)
    ON CONFLICT (puzzle_date) DO NOTHING;
  $$
);
