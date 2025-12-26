-- ============================================================================
-- Fresh Schema for ES256 JWT + RLS Enforcement
-- ============================================================================
-- This migration creates a complete fresh schema optimized for ES256 JWT
-- authentication with PostgreSQL Row Level Security (RLS) enforcement.
--
-- Key Changes from Previous Schema:
-- - RLS policies designed to work with auth.uid() set via SET LOCAL
-- - No application-level ownership checks needed
-- - All authorization handled at database level
-- ============================================================================

-- ============================================================================
-- DROP EXISTING TABLES (Fresh Start)
-- ============================================================================

-- Drop tables in correct order (respecting foreign key dependencies)
DROP TABLE IF EXISTS route_point_changes CASCADE;
DROP TABLE IF EXISTS route_proposals CASCADE;
DROP TABLE IF EXISTS route_editing_sessions CASCADE;
DROP TABLE IF EXISTS route_versions CASCADE;
DROP TABLE IF EXISTS routes CASCADE;
DROP TABLE IF EXISTS curated_tracks CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;
DROP FUNCTION IF EXISTS cleanup_stale_editing_sessions() CASCADE;
DROP FUNCTION IF EXISTS cleanup_old_point_changes() CASCADE;

-- Enable required extensions (after dropping tables)
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- CORE TABLES
-- ============================================================================

-- Curated Tracks Table
-- Stores verified off-road tracks with confidence scoring
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
CREATE INDEX idx_curated_tracks_source ON curated_tracks(source);
CREATE INDEX idx_curated_tracks_confidence ON curated_tracks(confidence);
CREATE INDEX idx_curated_tracks_region ON curated_tracks(region);

COMMENT ON TABLE curated_tracks IS 'Curated collection of off-road tracks from various sources';
COMMENT ON COLUMN curated_tracks.confidence IS 'Confidence level: 1=estimated, 2=satellite, 3=OSM, 4=community, 5=rally verified';

-- Routes Table
-- Stores user-created routes (metadata only, geometry in route_versions)
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    owner_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_routes_owner_id ON routes(owner_id);
CREATE INDEX idx_routes_updated_at ON routes(updated_at DESC);

COMMENT ON TABLE routes IS 'User-created routes - metadata only';
COMMENT ON COLUMN routes.owner_id IS 'UUID of the user who owns this route (from JWT sub claim)';

-- Route Versions Table
-- Stores complete version history of route geometries
CREATE TABLE route_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    geometry JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_route_versions_route_id ON route_versions(route_id);
CREATE INDEX idx_route_versions_created_at ON route_versions(created_at DESC);
CREATE INDEX idx_route_versions_route_created ON route_versions(route_id, created_at DESC);

COMMENT ON TABLE route_versions IS 'Complete version history for route geometries (immutable)';

-- ============================================================================
-- COLLABORATIVE EDITING TABLES
-- ============================================================================

-- Route Editing Sessions Table
-- Tracks active users editing a route (for presence indicators)
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

CREATE INDEX idx_route_editing_sessions_route_id ON route_editing_sessions(route_id);
CREATE INDEX idx_route_editing_sessions_last_heartbeat ON route_editing_sessions(last_heartbeat);
CREATE INDEX idx_route_editing_sessions_user_id ON route_editing_sessions(user_id);

COMMENT ON TABLE route_editing_sessions IS 'Active editing sessions for real-time collaboration and presence tracking';
COMMENT ON COLUMN route_editing_sessions.last_heartbeat IS 'Updated every 30s by client to show active presence';

-- Route Point Changes Table
-- Stores individual point movements as suggestions before acceptance
CREATE TABLE route_point_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    user_email TEXT NOT NULL,

    -- Point identification in MultiLineString
    feature_index INTEGER NOT NULL,
    point_index INTEGER NOT NULL,

    -- Original and new positions (GeoJSON coordinates: [lng, lat])
    original_position JSONB NOT NULL,
    new_position JSONB NOT NULL,

    -- Status tracking
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected')),

    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolved_by UUID
);

CREATE INDEX idx_route_point_changes_route_id ON route_point_changes(route_id);
CREATE INDEX idx_route_point_changes_status ON route_point_changes(status);
CREATE INDEX idx_route_point_changes_user_id ON route_point_changes(user_id);
CREATE INDEX idx_route_point_changes_route_status ON route_point_changes(route_id, status);

COMMENT ON TABLE route_point_changes IS 'Point-level change suggestions for collaborative editing';
COMMENT ON COLUMN route_point_changes.feature_index IS 'Index of feature in MultiLineString (0-based)';
COMMENT ON COLUMN route_point_changes.point_index IS 'Index of point within the LineString (0-based)';

-- Route Proposals Table
-- Stores full geometry proposals (different from point-level changes)
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

CREATE INDEX idx_route_proposals_route_id ON route_proposals(route_id);
CREATE INDEX idx_route_proposals_status ON route_proposals(status);
CREATE INDEX idx_route_proposals_created_by ON route_proposals(created_by);

COMMENT ON TABLE route_proposals IS 'Full geometry proposals - different from point-level changes';

-- ============================================================================
-- ROW LEVEL SECURITY (RLS) POLICIES
-- ============================================================================

-- Enable RLS on all tables
ALTER TABLE curated_tracks ENABLE ROW LEVEL SECURITY;
ALTER TABLE routes ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_editing_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_point_changes ENABLE ROW LEVEL SECURITY;
ALTER TABLE route_proposals ENABLE ROW LEVEL SECURITY;

-- Curated Tracks: Public read-only
CREATE POLICY "Public read curated tracks"
ON curated_tracks
FOR SELECT
USING (true);

-- Routes: Owner-based CRUD
CREATE POLICY "Users read own routes"
ON routes FOR SELECT
USING (owner_id = auth.uid());

CREATE POLICY "Users create routes"
ON routes FOR INSERT
WITH CHECK (owner_id = auth.uid());

CREATE POLICY "Users update own routes"
ON routes FOR UPDATE
USING (owner_id = auth.uid());

CREATE POLICY "Users delete own routes"
ON routes FOR DELETE
USING (owner_id = auth.uid());

-- Route Versions: Owner via parent route (immutable)
CREATE POLICY "Owner read route versions"
ON route_versions FOR SELECT
USING (
    EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_versions.route_id
          AND routes.owner_id = auth.uid()
    )
);

CREATE POLICY "Owner create route versions"
ON route_versions FOR INSERT
WITH CHECK (
    EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_versions.route_id
          AND routes.owner_id = auth.uid()
    )
);

-- Route Editing Sessions: Presence tracking
CREATE POLICY "Users see sessions for owned or joined routes"
ON route_editing_sessions FOR SELECT
USING (
    user_id = auth.uid()
    OR EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_editing_sessions.route_id
          AND routes.owner_id = auth.uid()
    )
);

CREATE POLICY "Users manage own session"
ON route_editing_sessions FOR ALL
USING (user_id = auth.uid())
WITH CHECK (user_id = auth.uid());

-- Route Point Changes: Collaborative suggestions
CREATE POLICY "Users read point changes for owned routes or own changes"
ON route_point_changes FOR SELECT
USING (
    user_id = auth.uid()
    OR EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_point_changes.route_id
          AND routes.owner_id = auth.uid()
    )
);

CREATE POLICY "Users create point changes"
ON route_point_changes FOR INSERT
WITH CHECK (user_id = auth.uid());

CREATE POLICY "Owners update point changes"
ON route_point_changes FOR UPDATE
USING (
    EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_point_changes.route_id
          AND routes.owner_id = auth.uid()
    )
);

-- Route Proposals: Full geometry proposals
CREATE POLICY "Users read proposals for owned routes or created ones"
ON route_proposals FOR SELECT
USING (
    created_by = auth.uid()
    OR EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_proposals.route_id
          AND routes.owner_id = auth.uid()
    )
);

CREATE POLICY "Users create proposals"
ON route_proposals FOR INSERT
WITH CHECK (created_by = auth.uid());

CREATE POLICY "Owners update proposals"
ON route_proposals FOR UPDATE
USING (
    EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_proposals.route_id
          AND routes.owner_id = auth.uid()
    )
);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers
CREATE TRIGGER update_curated_tracks_updated_at
    BEFORE UPDATE ON curated_tracks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_routes_updated_at
    BEFORE UPDATE ON routes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_route_proposals_updated_at
    BEFORE UPDATE ON route_proposals
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_route_point_changes_updated_at
    BEFORE UPDATE ON route_point_changes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Cleanup stale editing sessions (sessions without heartbeat for > 5 minutes)
CREATE OR REPLACE FUNCTION cleanup_stale_editing_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM route_editing_sessions
    WHERE last_heartbeat < NOW() - INTERVAL '5 minutes';

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_stale_editing_sessions IS 'Remove editing sessions without heartbeat for 5+ minutes';

-- Auto-archive old point changes (accepted/rejected changes older than 24 hours)
CREATE OR REPLACE FUNCTION cleanup_old_point_changes()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM route_point_changes
    WHERE status IN ('accepted', 'rejected')
    AND resolved_at < NOW() - INTERVAL '24 hours';

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_point_changes IS 'Remove resolved point changes older than 24 hours';

-- ============================================================================
-- SAMPLE DATA
-- ============================================================================

INSERT INTO curated_tracks (geometry, source, surface, confidence, region) VALUES
    (ST_GeomFromText('LINESTRING(-5.0 20.0, -5.1 20.1, -5.2 20.2)', 4326), 'rally', 'sand', 5, 'Western Sahara'),
    (ST_GeomFromText('LINESTRING(-4.5 20.5, -4.6 20.6)', 4326), 'osm', 'gravel', 3, 'Western Sahara'),
    (ST_GeomFromText('LINESTRING(-5.5 19.5, -5.6 19.6, -5.7 19.7)', 4326), 'curated', 'dirt', 4, 'Mauritania');

-- ============================================================================
-- REALTIME PUBLICATION (Enable Supabase Realtime)
-- ============================================================================

-- Enable realtime for collaborative editing tables
ALTER PUBLICATION supabase_realtime ADD TABLE route_editing_sessions;
ALTER PUBLICATION supabase_realtime ADD TABLE route_point_changes;
ALTER PUBLICATION supabase_realtime ADD TABLE route_versions;
ALTER PUBLICATION supabase_realtime ADD TABLE routes;

-- ============================================================================
-- SECURITY NOTES
-- ============================================================================

-- IMPORTANT: RLS policies use auth.uid() which relies on JWT claims being set
-- in the transaction context via:
--
--   SET LOCAL role authenticated;
--   SET LOCAL "request.jwt.claim.sub" TO '<user-uuid>';
--
-- The Rust backend handles this automatically via RlsTransaction wrapper.
--
-- All authorization is enforced at the database level - no application-level
-- ownership checks are needed or performed.
