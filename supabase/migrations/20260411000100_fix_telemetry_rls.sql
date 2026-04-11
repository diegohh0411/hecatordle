-- Fix RLS policies for player_telemetry to support upsert
-- Upsert requires SELECT, INSERT, and UPDATE permissions.

DROP POLICY IF EXISTS "Allow public inserts to player_telemetry" ON player_telemetry;
DROP POLICY IF EXISTS "Allow public updates to player_telemetry" ON player_telemetry;

CREATE POLICY "Allow public all access to player_telemetry"
ON player_telemetry
FOR ALL 
TO anon
USING (true)
WITH CHECK (true);
