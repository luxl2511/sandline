-- Add control_points column to routes table
ALTER TABLE routes ADD COLUMN control_points JSONB;
