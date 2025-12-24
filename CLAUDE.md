# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dakar Planner is a collaborative route planning application for off-road and rally racing. The application uses curated track data with confidence scoring and enables collaborative route proposals.

**Architecture:**
- Frontend: Next.js 14 (App Router) + TypeScript + Mapbox GL JS
- Backend: Rust + Axum web framework
- Database: PostgreSQL 15 + PostGIS (hosted on Supabase)

## Development Commands

### Frontend (`/frontend`)
```bash
npm run dev      # Start dev server on http://localhost:3000
npm run build    # Production build
npm run lint     # Run ESLint
```

### Backend (`/backend`)
```bash
cargo run        # Start server on http://localhost:8080
cargo build      # Build release binary
cargo test       # Run tests
```

### Local Database
```bash
docker-compose up -d     # Start PostgreSQL + PostGIS
docker-compose down      # Stop database
```

### Deployment
Deployments run from `main` branch only. If on a feature branch:
```bash
./scripts/merge-to-main.sh    # Merge to main
./scripts/quick-deploy.sh     # Interactive deployment wizard
```

## Core Architecture Patterns

### Frontend State Management

**Zustand Store** (`frontend/src/lib/store.ts`):
- Global state for map layers, selected routes, proposals, and drawing state
- Actions: `toggleLayer`, `setSelectedRoute`, `startDrawing`, `stopDrawing`, etc.
- Usage: `const { selectedRoute, setSelectedRoute } = useMapStore()`

**Auth Context** (`frontend/src/contexts/AuthContext.tsx`):
- Wraps entire app in `layout.tsx`
- Provides: `user`, `session`, `signIn`, `signUp`, `signOut`
- Auto-attaches JWT tokens to all API requests via axios interceptor

**Real-time Updates**:
- `useRealtimeRoutes(options)` - Subscribe to route table changes
- `useRealtimeProposals(routeId)` - Subscribe to proposal changes for a route
- Uses Supabase Realtime with WebSocket connections

### Backend Request Flow

1. **Request enters** → `main.rs` router
2. **CORS validation** → Supports wildcard domains (e.g., `*.vercel.app`)
3. **Auth middleware** → `middleware/auth.rs` validates JWT for protected routes
4. **Route handler** → `routes/route_handlers.rs` or `routes/proposals.rs`
5. **Database query** → SQLx with compile-time checked queries
6. **Response** → JSON serialization via serde

**Protected Routes** (require `AuthUser` extractor):
- `POST /api/routes` - Create route
- `PUT /api/routes/:id` - Update route (ownership verified)
- `POST /api/proposals` - Create proposal

**Public Routes**:
- `GET /api/routes` - List all routes
- `GET /api/routes/:id` - Get single route
- `GET /api/tracks` - List curated tracks

### Authentication Flow

**Frontend:**
1. User clicks "Sign In" → `AuthModal` appears
2. Credentials submitted → `AuthContext.signIn()` → Supabase Auth
3. Session stored → JWT token in `supabase.auth.getSession()`
4. All API requests → `api.ts` interceptor adds `Authorization: Bearer <token>`

**Backend:**
1. Request with `Authorization` header
2. `AuthUser` extractor validates JWT using `SUPABASE_JWT_SECRET`
3. Extracts user ID from token claims
4. Handler receives `auth_user: AuthUser` parameter

### Database Schema Critical Points

**Routes + Versioning:**
- `routes` table: Core route metadata (id, name, owner_id)
- `route_versions` table: Geometry history (one-to-many with routes)
- Latest version fetched via `LATERAL` join in queries

**Geometry Storage:**
- All geometries stored as `geometry(MultiLineString, 4326)` in PostGIS
- Frontend sends GeoJSON → Backend converts → PostGIS stores
- Queries use `ST_AsGeoJSON()` for returning geometry to frontend

**Confidence Scoring:**
- `curated_tracks.confidence`: Integer 1-5
- Maps to color scale in `tailwind.config.js`:
  - 5: Green (#22c55e)
  - 4: Lime (#84cc16)
  - 3: Yellow (#eab308)
  - 2: Orange (#f97316)
  - 1: Red (#ef4444)

### Map Drawing Workflow

1. User clicks "Create New Route" → `RouteEditor.tsx`
2. Check auth → If not logged in, show `AuthModal`
3. If logged in → `startDrawing()` in Zustand store
4. `MapView.tsx` detects `isDrawing` → Activates `useMapboxDraw` hook
5. User draws on map → Mapbox Draw emits events
6. `onDrawCreate` callback → `setDrawnGeometry()` in store
7. `RouteEditor` detects geometry → Opens `RouteCreationDialog`
8. User enters name → `createRoute()` API call
9. Backend stores route → Supabase Realtime broadcasts change
10. All connected clients receive update via `useRealtimeRoutes`

### Environment Variables

**Frontend** (`.env.local`):
- `NEXT_PUBLIC_MAPBOX_TOKEN` - Mapbox GL JS access token
- `NEXT_PUBLIC_API_URL` - Backend API URL
- `NEXT_PUBLIC_SUPABASE_URL` - Supabase project URL
- `NEXT_PUBLIC_SUPABASE_ANON_KEY` - Supabase anon/public key

**Backend** (`.env`):
- `DATABASE_URL` - PostgreSQL connection string
- `SUPABASE_JWT_SECRET` - Secret for validating Supabase JWTs (from Supabase Dashboard → Settings → API → JWT Secret)
- `HOST` - Server bind address (default: 0.0.0.0)
- `PORT` - Server port (default: 8080)
- `ALLOWED_ORIGINS` - Comma-separated CORS origins (supports wildcards: `*.vercel.app`)

### Dark Mode Implementation

- Tailwind configured with `darkMode: 'media'` - uses system preference
- All components use `dark:` variants (e.g., `bg-white dark:bg-gray-800`)
- No manual theme toggle - follows OS/browser settings automatically

### Key File Locations

**Frontend:**
- `src/lib/api.ts` - API client with auth interceptor
- `src/lib/store.ts` - Zustand state management
- `src/lib/supabase.ts` - Supabase client instance
- `src/components/map/MapView.tsx` - Main map component
- `src/hooks/useMapboxDraw.ts` - Mapbox Draw integration
- `src/contexts/AuthContext.tsx` - Authentication provider

**Backend:**
- `src/main.rs` - Server entry point, CORS config
- `src/routes/mod.rs` - API route definitions
- `src/middleware/auth.rs` - JWT validation
- `src/models/*.rs` - Database models
- `src/db/mod.rs` - Connection pool setup

### Testing Real-time Features

To test Supabase Realtime:
1. Ensure `routes` and `route_proposals` tables have replication enabled in Supabase Dashboard
2. Open app in two browser windows
3. Create a route in one window
4. Should appear immediately in other window without refresh

### Common Gotchas

**PostGIS Geometry Handling:**
- Frontend sends GeoJSON `MultiLineString`
- Backend receives as `serde_json::Value` (geometry field)
- Database stores as PostGIS `geometry(MultiLineString, 4326)`
- Always use `ST_AsGeoJSON()` when querying geometry back to frontend

**CORS Wildcard Support:**
- Custom CORS predicate in `main.rs` supports patterns like `*.vercel.app`
- Required for Vercel preview deployments
- Must include `Authorization` header in `allow_headers`

**Auth Extractor Order:**
- In Axum handlers, `AuthUser` must come BEFORE `State<DbPool>` and `Json<T>`
- Correct: `auth_user: AuthUser, State(pool): State<DbPool>, Json(payload): Json<T>`
- Incorrect order will cause compilation errors

**Supabase JWT Validation:**
- JWT audience must be `["authenticated"]` in validation
- Algorithm is `HS256`
- User ID is in `claims.sub` field as string (convert to UUID for database)
