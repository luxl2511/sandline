-- ============================================================================
-- Dakar Planner Consolidated Schema
-- ============================================================================

-- Fresh Start
DROP TABLE IF EXISTS route_point_changes CASCADE;
DROP TABLE IF EXISTS route_proposals CASCADE;
DROP TABLE IF EXISTS route_editing_sessions CASCADE;
DROP TABLE IF EXISTS route_versions CASCADE;
DROP TABLE IF EXISTS routes CASCADE;
DROP TABLE IF EXISTS curated_tracks CASCADE;

-- Extensions
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Curated Tracks Table
CREATE TABLE curated_tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    geometry GEOMETRY(LineString, 4326) NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('osm', 'rally', 'curated')),
    surface TEXT,
    confidence INTEGER NOT NULL CHECK (confidence BETWEEN 1 AND 5),
    last_verified DATE,
    region TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_curated_tracks_geometry ON curated_tracks USING GIST(geometry);

-- Routes Table (Metadata + Control Points)
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    owner_id UUID NOT NULL,
    control_points JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_routes_owner_id ON routes(owner_id);

-- Route Versions Table (Calculated Geometry + Stats)
CREATE TABLE route_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    geometry JSONB NOT NULL,
    length_km DOUBLE PRECISION,
    estimated_time_min INTEGER,
    created_by UUID, -- The user who generated this version
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_route_versions_route_id ON route_versions(route_id);
CREATE INDEX idx_route_versions_created_at ON route_versions(created_at DESC);

-- Collaborative Editing Sessions
CREATE TABLE route_editing_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    user_email TEXT NOT NULL,
    user_avatar_url TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_heartbeat TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(route_id, user_id)
);

-- Route Point Changes (Suggestions)
CREATE TABLE route_point_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    user_email TEXT NOT NULL,
    feature_index INTEGER NOT NULL,
    point_index INTEGER NOT NULL,
    original_position JSONB NOT NULL,
    new_position JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolved_by UUID
);

-- Route Proposals (Full Geometry)
CREATE TABLE route_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    geometry JSONB NOT NULL,
    comment TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected')),
    created_by UUID,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_curated_tracks_updated_at BEFORE UPDATE ON curated_tracks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_routes_updated_at BEFORE UPDATE ON routes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_route_point_changes_updated_at BEFORE UPDATE ON route_point_changes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_route_proposals_updated_at BEFORE UPDATE ON route_proposals FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Row Level Security (RLS)
ALTER TABLE curated_tracks ENABLE ROW LEVEL SECURITY;
ALTER TABLE routes ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_editing_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_point_changes ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_proposals ENABLE ROW LEVEL SECURITY;

-- Policies
CREATE POLICY "Public read curated tracks" ON curated_tracks FOR SELECT USING (true);
CREATE POLICY "Authenticated users view all routes" ON routes FOR SELECT TO authenticated USING (true);
CREATE POLICY "Users create own routes" ON routes FOR INSERT TO authenticated WITH CHECK (owner_id = auth.uid());
CREATE POLICY "Users update own routes" ON routes FOR UPDATE TO authenticated USING (owner_id = auth.uid());
CREATE POLICY "Users delete own routes" ON routes FOR DELETE TO authenticated USING (owner_id = auth.uid());

CREATE POLICY "Authenticated users view all route versions" ON route_versions FOR SELECT TO authenticated USING (true);
CREATE POLICY "Owner create route versions" ON route_versions FOR INSERT TO authenticated WITH CHECK (EXISTS (SELECT 1 FROM routes WHERE id = route_id AND owner_id = auth.uid()));

CREATE POLICY "Users manage own session" ON route_editing_sessions FOR ALL TO authenticated USING (user_id = auth.uid()) WITH CHECK (user_id = auth.uid());
CREATE POLICY "Authenticated users view all sessions" ON route_editing_sessions FOR SELECT TO authenticated USING (true);

CREATE POLICY "Authenticated users view all point changes" ON route_point_changes FOR SELECT TO authenticated USING (true);
CREATE POLICY "Users create point changes" ON route_point_changes FOR INSERT TO authenticated WITH CHECK (user_id = auth.uid());
CREATE POLICY "Owners update point changes" ON route_point_changes FOR UPDATE TO authenticated USING (EXISTS (SELECT 1 FROM routes WHERE id = route_id AND owner_id = auth.uid()));

CREATE POLICY "Authenticated users view all proposals" ON route_proposals FOR SELECT TO authenticated USING (true);
CREATE POLICY "Users create proposals" ON route_proposals FOR INSERT TO authenticated WITH CHECK (created_by = auth.uid());
CREATE POLICY "Owners update proposals" ON route_proposals FOR UPDATE TO authenticated USING (EXISTS (SELECT 1 FROM routes WHERE id = route_id AND owner_id = auth.uid()));

-- Realtime Replication
ALTER TABLE routes REPLICA IDENTITY FULL;
ALTER TABLE route_versions REPLICA IDENTITY FULL;
ALTER TABLE route_point_changes REPLICA IDENTITY FULL;
ALTER TABLE route_editing_sessions REPLICA IDENTITY FULL;

-- Check if publication exists before adding tables
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'supabase_realtime') THEN
        CREATE PUBLICATION supabase_realtime;
    END IF;
END $$;

ALTER PUBLICATION supabase_realtime ADD TABLE routes;
ALTER PUBLICATION supabase_realtime ADD TABLE route_versions;
ALTER PUBLICATION supabase_realtime ADD TABLE route_point_changes;
ALTER PUBLICATION supabase_realtime ADD TABLE route_editing_sessions;

-- Permissions
GRANT ALL ON ALL TABLES IN SCHEMA public TO postgres, service_role;
GRANT SELECT ON curated_tracks TO anon, authenticated;
GRANT SELECT, INSERT, UPDATE, DELETE ON routes TO authenticated;
GRANT SELECT, INSERT ON route_versions TO authenticated;
GRANT SELECT, INSERT, UPDATE ON route_point_changes TO authenticated;
GRANT SELECT, INSERT, UPDATE, DELETE ON route_editing_sessions TO authenticated;
GRANT SELECT, INSERT, UPDATE ON route_proposals TO authenticated;

-- Sample Data for testing
INSERT INTO curated_tracks (geometry, source, surface, confidence, region) VALUES
    (ST_GeomFromText('LINESTRING(-5.0 20.0, -5.1 20.1, -5.2 20.2)', 4326), 'rally', 'sand', 5, 'Western Sahara'),
    (ST_GeomFromText('LINESTRING(-4.5 20.5, -4.6 20.6)', 4326), 'osm', 'gravel', 3, 'Western Sahara'),
    (ST_GeomFromText('LINESTRING(-5.5 19.5, -5.6 19.6, -5.7 19.7)', 4326), 'curated', 'dirt', 4, 'Mauritania');
