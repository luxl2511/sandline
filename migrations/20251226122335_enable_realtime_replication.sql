-- ============================================================================
-- Enable Realtime Replication for Collaborative Features
-- ============================================================================
-- This migration enables PostgreSQL replication for Supabase Realtime.
-- Allows clients to subscribe to database changes in real-time.
--
-- Tables enabled:
-- - routes: Route list updates
-- - route_versions: Route geometry updates
-- - route_proposals: Collaborative proposal updates
-- - route_point_changes: Point change suggestions
-- ============================================================================

-- Enable replication identity FULL for routes table
-- Allows Realtime to broadcast all columns on INSERT/UPDATE/DELETE
ALTER TABLE routes REPLICA IDENTITY FULL;

-- Enable replication identity FULL for route_versions table
-- Critical for seeing geometry updates in real-time
ALTER TABLE route_versions REPLICA IDENTITY FULL;

-- Enable replication identity FULL for route_proposals table
-- Allows collaborative proposal workflow
ALTER TABLE route_proposals REPLICA IDENTITY FULL;

-- Enable replication identity FULL for route_point_changes table
-- Enables real-time point change notifications
ALTER TABLE route_point_changes REPLICA IDENTITY FULL;

-- Optional: Enable for editing sessions (for advanced presence features)
ALTER TABLE route_editing_sessions REPLICA IDENTITY FULL;

COMMENT ON TABLE routes IS 'Realtime enabled: clients can subscribe to route list changes';
COMMENT ON TABLE route_versions IS 'Realtime enabled: clients can subscribe to geometry updates';
COMMENT ON TABLE route_proposals IS 'Realtime enabled: clients can subscribe to proposal changes';
COMMENT ON TABLE route_point_changes IS 'Realtime enabled: clients can subscribe to point change suggestions';
