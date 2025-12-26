-- Drop the old, restrictive policy
DROP POLICY "Owner read route versions" ON route_versions;

-- Create a new, permissive policy that allows any authenticated user to see all route versions
CREATE POLICY "Allow all authenticated users to see all route versions"
ON route_versions
FOR SELECT
TO authenticated
USING (
    EXISTS (
        SELECT 1
        FROM routes
        WHERE routes.id = route_versions.route_id
    )
);
