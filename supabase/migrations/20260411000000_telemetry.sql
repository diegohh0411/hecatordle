-- Create player_telemetry table for anonymous telemetry
CREATE TABLE player_telemetry (
    player_id UUID PRIMARY KEY,
    
    -- Player Stats
    games_played INT DEFAULT 0,
    games_won INT DEFAULT 0,
    current_streak INT DEFAULT 0,
    max_streak INT DEFAULT 0,
    last_puzzle_date DATE,
    guess_distribution JSONB DEFAULT '{"1-20":0,"21-40":0,"41-60":0,"61-80":0,"81-100":0,"101-120":0,"121-134":0}'::jsonb,
    
    -- Metadata
    timezone TEXT,
    language TEXT,
    user_agent TEXT,
    screen_width INT,
    screen_height INT,
    
    -- Live Progress
    current_game_guesses INT DEFAULT 0,
    current_game_solved INT DEFAULT 0,
    
    -- Timestamps
    last_seen_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enable RLS
ALTER TABLE player_telemetry ENABLE ROW LEVEL SECURITY;

-- Create policies for anonymous access
-- Note: In a production app, we would ideally use authenticated sessions.
-- For this anonymous wordle clone, we allow anyone with the anon key to upsert.
CREATE POLICY "Allow public inserts to player_telemetry"
ON player_telemetry FOR INSERT
TO anon
WITH CHECK (true);

CREATE POLICY "Allow public updates to player_telemetry"
ON player_telemetry FOR UPDATE
TO anon
USING (true);
