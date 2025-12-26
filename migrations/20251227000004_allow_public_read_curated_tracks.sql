-- Allow public read access to curated_tracks table
-- Curated tracks are reference data used for routing, should be readable by all users

CREATE POLICY "Anyone can view curated tracks"
ON curated_tracks
FOR SELECT
TO authenticated, anon
USING (true);
