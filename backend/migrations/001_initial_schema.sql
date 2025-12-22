-- Enable PostGIS extension
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Curated Tracks Table
-- Stores all verified and curated off-road tracks
CREATE TABLE curated_tracks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    geometry GEOMETRY(LineString, 4326) NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('osm', 'rally', 'curated')),
    surface TEXT,
    confidence INTEGER NOT NULL CHECK (confidence BETWEEN 1 AND 5),
    last_verified DATE,
    region TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Spatial index for fast queries
CREATE INDEX idx_curated_tracks_geometry ON curated_tracks USING GIST(geometry);
CREATE INDEX idx_curated_tracks_source ON curated_tracks(source);
CREATE INDEX idx_curated_tracks_confidence ON curated_tracks(confidence);
CREATE INDEX idx_curated_tracks_region ON curated_tracks(region);

-- Routes Table
-- Stores user-created routes
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    owner_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_routes_owner_id ON routes(owner_id);

-- Route Versions Table
-- Stores version history of routes
CREATE TABLE route_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    route_id UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    geometry JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_route_versions_route_id ON route_versions(route_id);
CREATE INDEX idx_route_versions_created_at ON route_versions(created_at DESC);

-- Route Proposals Table
-- Stores collaborative route proposals
CREATE TABLE route_proposals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
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

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to tables
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

-- Sample data for testing
INSERT INTO curated_tracks (geometry, source, surface, confidence, region) VALUES
    (ST_GeomFromText('LINESTRING(-5.0 20.0, -5.1 20.1, -5.2 20.2)', 4326), 'rally', 'sand', 5, 'Western Sahara'),
    (ST_GeomFromText('LINESTRING(-4.5 20.5, -4.6 20.6)', 4326), 'osm', 'gravel', 3, 'Western Sahara'),
    (ST_GeomFromText('LINESTRING(-5.5 19.5, -5.6 19.6, -5.7 19.7)', 4326), 'curated', 'dirt', 4, 'Mauritania');

COMMENT ON TABLE curated_tracks IS 'Curated collection of off-road tracks from various sources';
COMMENT ON TABLE routes IS 'User-created routes';
COMMENT ON TABLE route_versions IS 'Version history for routes';
COMMENT ON TABLE route_proposals IS 'Collaborative proposals for route modifications';

COMMENT ON COLUMN curated_tracks.confidence IS 'Confidence level: 1=estimated, 2=satellite, 3=OSM, 4=community, 5=rally verified';
COMMENT ON COLUMN curated_tracks.source IS 'Data source: osm, rally, or curated';
