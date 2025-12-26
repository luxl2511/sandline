-- Core CRUD privileges for authenticated users
GRANT SELECT, INSERT, UPDATE, DELETE ON routes TO authenticated;
GRANT SELECT, INSERT ON route_versions TO authenticated;
GRANT SELECT, INSERT, UPDATE ON route_point_changes TO authenticated;
GRANT SELECT, INSERT, UPDATE ON route_proposals TO authenticated;
GRANT SELECT, INSERT, UPDATE, DELETE ON route_editing_sessions TO authenticated;

-- Read-only access
GRANT SELECT ON curated_tracks TO authenticated;
