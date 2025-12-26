-- Drop the old, restrictive policy
DROP POLICY "Users read own routes" ON routes;

-- Create a new, permissive policy that allows any authenticated user to see all routes
CREATE POLICY "Allow all authenticated users to see all routes"
ON routes
FOR SELECT
TO authenticated
USING (true);
