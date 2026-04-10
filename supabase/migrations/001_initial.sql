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
