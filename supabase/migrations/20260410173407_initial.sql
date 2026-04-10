-- Create word_bank table
CREATE TABLE word_bank (
    word VARCHAR(5) PRIMARY KEY,
    frequency_per_1k FLOAT NOT NULL,
    difficulty VARCHAR(10) DEFAULT 'normal',
    sources JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create daily_puzzles table
CREATE TABLE daily_puzzles (
    puzzle_date DATE PRIMARY KEY,
    words JSONB NOT NULL,
    generated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enable RLS
ALTER TABLE word_bank ENABLE ROW LEVEL SECURITY;
ALTER TABLE daily_puzzles ENABLE ROW LEVEL SECURITY;

-- Create read-only policies for anonymous access
CREATE POLICY "Allow public read-only access to word_bank"
ON word_bank FOR SELECT
TO anon
USING (true);

CREATE POLICY "Allow public read-only access to daily_puzzles"
ON daily_puzzles FOR SELECT
TO anon
USING (true);

-- Enable pg_cron extension
CREATE EXTENSION IF NOT EXISTS pg_cron;

-- Schedule daily puzzle generation at midnight UTC
SELECT cron.schedule('generate-daily-puzzle',
  '0 0 * * *',
  $$
    INSERT INTO daily_puzzles (puzzle_date, words)
    SELECT
      CURRENT_DATE,
      (SELECT jsonb_agg(word) FROM (
        (SELECT word FROM word_bank WHERE difficulty = 'easy'   ORDER BY RANDOM() LIMIT 51)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'normal' ORDER BY RANDOM() LIMIT 51)
        UNION ALL
        (SELECT word FROM word_bank WHERE difficulty = 'hard'   ORDER BY RANDOM() LIMIT 26)
      ) sub)
    ON CONFLICT (puzzle_date) DO NOTHING;
  $$
);
