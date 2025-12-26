# GEMINI.md

This file provides guidance to Gemini when working with code in this repository.

## Project Overview

Dakar Planner is a collaborative route planning application for off-road and rally racing. The application uses curated track data with confidence scoring and enables collaborative route proposals.

**Architecture:**

- Frontend: Next.js 14 (App Router) + TypeScript + Mapbox GL JS
- Backend: Rust + Axum web framework
- Database: PostgreSQL 15 + PostGIS (hosted on Supabase)

## User Experience (UI/UX) and Interaction (Current State)

The application aims for a highly interactive and collaborative route planning experience, moving away from a distinct "editing mode" towards always-on interactive elements.

### Core User Stories & Interactions

1. **User Authentication:** Users are prompted to authenticate (register/login) upon opening the app. This is a prerequisite for most interactive features.
2. **Route Display:** All available routes are shown on the map by default.
    - Routes consist of a minimum of 2 control points.
    - Between these control points, there are calculated lines representing the path (either free routing or road routing). These lines are clickable.
3. **Control Point Interaction:**
    - **Dragging:** Users can drag control points.
      - If it's the user's _own_ route, dragging immediately recalculates the routing line segments before and after the dragged point, and updates the route directly.
      - If it's _another person's_ route, dragging a point creates a **proposal**. The proposed new route segment is shown in a different, perhaps greyed-out or dashed, color. A dotted line connects the original point to the proposed new point.
    - **Extending Route:** Clicking on the _first_ or _last_ control point of an existing route allows the user to extend it by adding new points, effectively continuing the drawing process from that end.
    - **Deleting Point:** When hovering over a control point, a small "x" (delete icon) appears, allowing the user to remove that specific point from the route.
4. **Segment Interaction:** Clicking on a routing line segment between control points opens a "comic bubble" popup above it.
    - This bubble provides options like choosing between "free routing" (straight line) and "road routing" for that specific segment.
    - It also displays basic segment statistics (e.g., approximate length, estimated travel time).
5. **Collaborative Real-time Feedback:**
    - **Concurrent Dragging:** If a control point is being dragged by another user, the current user cannot drag that specific point but can observe its movement in real-time. Once the other user releases the drag, the point becomes draggable again. (For a user's own route, this concurrency control is not applicable as they have direct control).
    - **Presence Indicators:** Map pins for control points include Google Docs-like presence information (e.g., a small person icon or initials if someone else is actively dragging it).
6. **Route Statistics Display:** Upon interacting with a route (dragging a point, clicking a segment), a dedicated panel appears in the top-right corner. This panel displays comprehensive statistics for the selected route:
    - Total kilometers
    - Estimated total travel time
    - Route owner
    - Last changed (timestamp)
    - Last changer (user identifier)

### Visual & Aesthetic Goals

- **Map Pins:** Control points are rendered as "map pins" rather than simple markers.
- **Gamy Feel:** The overall UI should have a "gamy" aesthetic, featuring big, animated map pins.
- **Responsive Dragging:** Drags are responsive, with instant feedback.
- **Animated Rerouting:** When a point is dragged and released, the recalculation of the route segment(s) is shown with an animation, where the route "appears" or "draws" itself from start to finish along the new path.

### Architectural Confirmations

- **Real-time Infrastructure:** Utilizes Supabase for real-time channels to broadcast cursor positions, drag events, and point changes.
- **Backend:** Powered by a Rust backend for processing geometry, handling authentication, and managing database interactions.
- **Frontend:** Built with Next.js and Mapbox GL JS for an interactive mapping interface.
- **Access Control:** The current setup assumes a single large user group where everyone sees everyone else's routes.

## Current Implementation Details (Updated)

### Frontend State Management

**Zustand Store** (`frontend/src/lib/store.ts`):

- Global state now manages `editingRouteId` to signify which route is currently the focus of collaborative interaction or detail viewing, rather than an explicit `isEditingRoute` boolean.
- `setEditingRouteId(routeId)`: Sets the currently "active" route for detail/collaboration.
- `clearEditingState()`: Resets the `editingRouteId` and related collaborative states.

**Auth Context** (`frontend/src/contexts/AuthContext.tsx`):

- (No changes - still handles user authentication and JWT management).

**Real-time Updates**:

- `useRealtimeRoutes(options)`: Subscribes to route table changes to keep the list of routes updated.
- `useRealtimeProposals(routeId)`: Subscribes to proposal changes for a specific route.
- `useLiveCursors(mapRef, routeId)`: Tracks and broadcasts live cursor positions. Now enabled based on `!!routeId`.
- `useRealtimeBroadcast(routeId)`: Generic hook for sending/receiving ephemeral broadcast messages (e.g., drag starts/updates/ends). Now enabled based on `!!routeId`.

### Backend Request Flow

(No major changes to overall flow, but specific handler logic has been updated for control points and proposals).

**Updated Route Handlers:**

- `POST /api/routes`: `create_route` now accepts `control_points` in the payload and stores them in the `routes` table.
- `PUT /api/routes/:id/control-points`: `update_route_control_points` handler:
  - Accepts `routeId`, new `controlPoints`, `featureIndex`, and `pointIndex`.
  - **Ownership Check:** Determines if the `auth_user.id` matches the route's `owner_id`.
  - **Direct Update (Owner):** If the user owns the route, it updates the `control_points` column in the `routes` table and creates a new `route_version` by re-processing the geometry from the updated control points.
  - **Proposal Creation (Non-Owner):** If the user does _not_ own the route, it creates a new `route_point_changes` entry with `status = 'pending'`, detailing the proposed change (original vs. new position). The actual route geometry is _not_ changed.

### Database Schema Critical Points

**New `control_points` column:**

- The `routes` table now includes a `control_points JSONB` column to store the user-defined control points for each route.

**Collaborative Proposals (`route_point_changes`):**

- `route_point_changes` table stores individual point movement suggestions. This is used when a non-owner drags a control point.

### Frontend Components & Interactions (Updated)

- **`MapView.tsx`**:
  - Now passes the `routes` array directly to `RouteRenderer` and `ControlPointsLayer`.
  - Manages `segmentBubbleInfo` state to display `SegmentOptionsBubble` on route segment clicks.
  - Integrates `RouteStatsPanel` for displaying route details when `editingRouteId` is active.
  - **Click-to-Create:** An `onClick` handler on `MapboxMap` checks if `isDrawing` is false and no map features were clicked; if so, it calls `startDrawing()`.
- **`RouteEditor.tsx`**:
  - Refactored from an "editing mode" toggler to a "Route Info Panel."
  - `isEditingRoute`, `startEditingRoute`, `stopEditingRoute` removed from `useMapStore` usage.
  - Now uses `editingRouteId` to determine which route's details to display.
  - Includes `PointChangeProposalList` to show pending proposals for the active route.
- **`RouteRenderer.tsx`**:
  - Accepts an `onRouteClick` prop to handle clicks on route line segments.
  - Uses `line-gradient` in Mapbox GL JS paint properties to animate route drawing.
- **`ControlPointsLayer.tsx`**:
  - Now renders individual `ControlPointPin` components for each control point.
  - Utilizes `useRealtimeBroadcast` to track active drags, making `Marker` draggable only if no other user is actively dragging that specific point.
  - **Extend Route:** `onClick` handlers on the first/last `Marker` (ControlPointPin) trigger `startDrawing()` and set the initial `drawnGeometry` to extend the route.
  - **Delete Route Point:** Passes an `onDelete` handler to `ControlPointPin` to remove the point from the route's `controlPoints` array and update the backend.
- **`ControlPointPin.tsx` (New Component)**:
  - Renders a visual "map pin" for each control point.
  - Displays presence indicators (user initials/email) if another user is dragging the point.
  - Includes a small "x" delete button that appears on hover, triggering `onDelete` from `ControlPointsLayer`.
- **`PointChangeProposalList.tsx` (New Component)**:
  - Displayed within `RouteEditor` when there are `pendingPointChanges` for the `editingRouteId`.
  - Shows details of each proposed point change.
  - Provides "Accept" and "Reject" buttons (visible only to the route owner) that call `updatePointChangeStatus` API.
- **`SegmentOptionsBubble.tsx` (New Component)**:
  - A `Popup` component that appears on clicking a route segment.
  - Currently displays placeholder segment length and estimated time.
  - Includes "Road Routing" and "Free Routing" buttons (functionality to be implemented).
- **`RouteStatsPanel.tsx` (New Component)**:
  - Displayed in the top-right corner of `MapView` when `editingRouteId` is active.
  - Shows route details like name, owner, length (placeholder), time (placeholder), last changed, and last changed by (placeholder).

## Testing Real-time Features

To test Supabase Realtime:

1. Ensure `routes`, `route_point_changes`, `route_editing_sessions`, and `route_versions` tables have replication enabled in Supabase Dashboard.
2. Open the app in two different browser windows/devices, logged in as different users.
3. **Collaborative Dragging:** Drag a control point on one user's route. The other user should see the live drag and be prevented from dragging that same point. If not the owner, a proposal should appear in the Route Editor.
4. **Proposal Acceptance/Rejection:** As the route owner, accept/reject a proposal. The map should update accordingly.

## Future Spatial Features (Not Yet Implemented / In Progress)

- **Road-Snapping Logic**
- **Terrain-Aware Distance Calculations**
- **Authoritative Spatial Pipeline**
- **Segment Routing Options:** Implement functionality for "Road Routing" and "Free Routing" buttons in `SegmentOptionsBubble`.
- **Accurate Route Statistics:** Replace placeholder data in `RouteStatsPanel` with real calculations.

## Testing Strategy

Sandline uses a comprehensive testing approach to ensure reliability and prevent regressions.

### Backend Testing (Rust + Cargo)

**Unit Tests**: Inline tests in source files for pure functions (geometry, validation, etc.)

- Run: `cargo test --lib`
- Location: `#[cfg(test)] mod tests` blocks in source files

**Integration Tests**: Full HTTP endpoint tests with real Supabase test database

- Run: `cargo test --test '*'`
- Location: `backend/tests/` directory
- Uses Supabase test project for realistic database testing

**Key Test Utilities**: `backend/tests/common/mod.rs`

- `create_test_pool()` - Test database connection
- `TestUser::create()` - Generate authenticated test users
- `send_authed_request()` - Helper for authenticated API calls

**Coverage Goal**: >70% code coverage, >90% for critical paths (auth, RLS, proposals)

### Frontend Testing (Vitest + React Testing Library)

**Unit Tests**: API client, Zustand store, utilities

- Run: `npm test`
- Location: `frontend/src/*/__tests__/*.test.{ts,tsx}`

**Component Tests**: React components with @testing-library/react

- Mocks: Mapbox GL JS, Supabase client, localStorage

**Hook Tests**: Custom hooks (real-time subscriptions, auth, map interactions)

**Coverage Goal**: >60% code coverage (Mapbox mocking complexity considered)

### E2E Testing (Playwright)

**Target Environment**: Staging (Vercel + Fly.io + Supabase test project)

- Run: `npx playwright test`
- Location: `e2e/` directory

**Multi-User Testing**: Playwright browser contexts simulate concurrent users

- Critical for testing real-time collaboration features

**Critical Flows Tested**:

1. User authentication (sign up, sign in, sign out)
2. Route creation with control points
3. Control point dragging (owner vs. non-owner behavior)
4. Proposal creation and acceptance/rejection
5. Real-time collaboration (live cursors, concurrent editing prevention)
6. Segment routing options (road vs. free)

**Test Users**: Pre-created test users in Supabase test project

- <test-user-1@example.com> (for owner scenarios)
- <test-user-2@example.com> (for collaborator scenarios)

### Running Tests Locally

**Backend**:

```bash
cd backend
# Unit tests only
cargo test --lib
# Integration tests (requires .env.test with Supabase test project)
cargo test --test '*'
```

**Frontend**:

```bash
cd frontend
npm test              # Run all tests
npm run test:ui       # Interactive UI
npm run test:coverage # Generate coverage report
```

**E2E**:

```bash
npx playwright test                    # Run all E2E tests
npx playwright test --ui               # Interactive mode
npx playwright test collaboration.spec # Run specific test file
```

### CI/CD Testing

**GitHub Actions Workflows**:

- **`.github/workflows/ci.yml`**: Runs backend + frontend tests on every PR
- **`.github/workflows/e2e-tests.yml`**: Runs E2E tests after staging deployments

**Coverage Reporting**: Codecov integration for tracking test coverage over time

## Error Monitoring & Observability

### Production Error Tracking

**Recommendation**: Integrate Sentry for error monitoring

**Backend (Rust)**:

- Add `sentry` crate for automatic panic/error capture
- Track: API errors, database errors, JWT validation failures, routing errors

**Frontend (Next.js)**:

- Add `@sentry/nextjs` for client-side and server-side errors
- Track: Mapbox errors, Supabase connection errors, authentication failures, API call failures

**Key Metrics to Monitor**:

- Error rate by endpoint
- JWT validation failures (potential security issues)
- Mapbox API failures (quota/rate limiting)
- Supabase Realtime connection drops
- Route recalculation errors

### Logging Strategy

**Backend**: Use `tracing` crate (already configured)

- Log levels: ERROR for critical issues, WARN for recoverable errors, INFO for key events

**Frontend**: Use `console.error` with structured logging

- Consider adding client-side logging service (e.g., LogRocket, Datadog RUM)

### Real-time Monitoring Considerations

**Critical to Monitor**:

- Supabase Realtime connection health
- WebSocket connection failures
- Concurrent editing conflicts
- Proposal processing delays
- JWKS cache refresh failures

### Future Enhancements

- **Performance Monitoring**: APM for backend response times (e.g., Datadog, New Relic)
- **User Analytics**: Track feature usage, route creation patterns (PostHog, Mixpanel)
- **Uptime Monitoring**: Synthetic checks for API endpoints (UptimeRobot, Pingdom)

### Common Gotchas

(Existing points still apply)

- `AuthUser` extractor should handle `sub` and `email` claims correctly from JWT.
- RLS policies are critical for proper authorization. `AuthUser` ensures `auth.uid()` and `auth.email()` are available within transactions.
