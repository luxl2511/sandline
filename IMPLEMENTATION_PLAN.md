# Dakar Planner - Implementation Plan

## 🎯 Project Vision

A data-curated, collaborative route planning platform for off-road and rally racing. Instead of manual GPX imports, users discover and plan routes using curated track data from OSM, rally archives, and community sources.

## 📋 Core Principles

1. **Tracks are First-Class Citizens** - Routes are built by selecting and combining existing tracks
2. **Data-Driven Planning** - Curate and integrate quality data sources
3. **Confidence Transparency** - Clear visualization of track reliability
4. **Collaborative Decision Making** - Proposals and discussions, not just sharing

---

## 🏗️ Architecture Overview

### Frontend Stack
- **Framework**: Next.js 14 (App Router)
- **Mapping**: Mapbox GL JS + react-map-gl
- **Drawing**: @mapbox/mapbox-gl-draw
- **State**: Zustand
- **Styling**: Tailwind CSS
- **Geo Utils**: @turf/turf

### Backend Stack
- **Framework**: Rust + Axum
- **Database**: PostgreSQL 15 + PostGIS 3.4
- **ORM**: SQLx
- **Data Format**: GeoJSON

### Infrastructure
- **Development**: Docker Compose
- **Database**: PostGIS container
- **Deployment**: TBD (Railway/Fly.io/VPS)

---

## 📊 Data Model

### Core Tables

#### `curated_tracks`
The heart of the system - all verified tracks from various sources.

```sql
- id: UUID
- geometry: GEOMETRY(LineString, 4326)
- source: TEXT ('osm' | 'rally' | 'curated')
- surface: TEXT (sand, gravel, dirt, mixed)
- confidence: INTEGER (1-5)
- last_verified: DATE
- region: TEXT
- created_at: TIMESTAMP
- updated_at: TIMESTAMP
```

**Confidence Levels:**
- 5 = Rally verified / Official race tracks
- 4 = Community verified / Multiple sources
- 3 = OSM track visible
- 2 = Satellite imagery traces
- 1 = Estimated / Unverified

#### `routes`
User-created routes (metadata only).

```sql
- id: UUID
- name: TEXT
- owner_id: UUID
- created_at: TIMESTAMP
- updated_at: TIMESTAMP
```

#### `route_versions`
Version history for routes (immutable).

```sql
- id: UUID
- route_id: UUID
- geometry: JSONB (MultiLineString)
- created_at: TIMESTAMP
```

#### `route_proposals`
Collaborative proposals for route changes.

```sql
- id: UUID
- route_id: UUID
- geometry: JSONB (MultiLineString)
- comment: TEXT
- status: TEXT ('pending' | 'accepted' | 'rejected')
- created_by: UUID
- created_at: TIMESTAMP
- updated_at: TIMESTAMP
```

---

## 🗺️ Data Sources

### 1. OpenStreetMap
**Priority Tags:**
- `highway=track`
- `highway=path`
- `highway=unclassified`
- `surface=sand|gravel|dirt|unpaved`
- `tracktype=grade2|grade3|grade4|grade5`

**Import Strategy:**
- Use Overpass API or PBF extracts
- Filter by region (bounding box)
- Initial confidence: 3
- Store as LineString geometries

**Tools:**
- `osmium` for PBF processing
- `osm2pgsql` alternative
- Custom Rust parser using `osmpbf` crate

### 2. Rally & Community Tracks
**Sources:**
- Dakar Rally historic routes (public archives)
- Overland community databases
- User-submitted verified routes

**Import Format:**
- GeoJSON
- KML (convert to GeoJSON)
- GPX (for historic rally data only)

**Quality Criteria:**
- Official sources: confidence 5
- Community verified: confidence 4
- User submitted: confidence 2-3

### 3. Satellite Imagery Integration
**PoC Approach:**
- Manual verification using satellite basemap
- Users draw/verify tracks visually
- Not automated detection (future enhancement)

---

## 🎨 Frontend Features

### Map View
**Layer Controls:**
- ✅ OSM Tracks (toggle)
- ✅ Curated Tracks (toggle)
- ✅ Satellite Basemap (toggle)
- ✅ My Routes (toggle)

**Visualization:**
- Track colors based on confidence level
- Line width based on zoom level
- Hover tooltips showing track metadata
- Different patterns for different surfaces

**Confidence Color Scheme:**
```
5: #22c55e (green-500)   - Rally verified
4: #84cc16 (lime-500)    - Community
3: #eab308 (yellow-500)  - OSM visible
2: #f97316 (orange-500)  - Satellite
1: #ef4444 (red-500)     - Estimated
```

### Route Editor
**Drawing Tools:**
- Line drawing mode
- Track snapping (snap to existing tracks)
- Edit mode (modify vertices)
- Delete mode

**Track Info:**
- Show which tracks are being used
- Display total distance
- Show confidence breakdown
- Surface type distribution

### Proposal System
**Create Proposal:**
- Draw alternative route
- Add comment/rationale
- Submit for review

**Review Proposals:**
- Side-by-side comparison
- Distance/confidence differences
- Accept/reject with feedback

---

## 🔧 Backend API

### Tracks Endpoints
```
GET  /api/tracks                 # List tracks (with filters)
GET  /api/tracks/:id             # Get track details
```

**Query Parameters:**
- `bbox`: Bounding box (minLon,minLat,maxLon,maxLat)
- `source`: Filter by source type
- `min_confidence`: Minimum confidence level
- `region`: Filter by region

### Routes Endpoints
```
GET  /api/routes                 # List user routes
GET  /api/routes/:id             # Get route with latest version
POST /api/routes                 # Create new route
PUT  /api/routes/:id             # Update route (creates new version)
```

### Proposals Endpoints
```
GET    /api/routes/:id/proposals  # List proposals for route
POST   /api/proposals             # Create proposal
PATCH  /api/proposals/:id         # Update proposal status
```

---

## 🤖 Data Ingestion Jobs

### OSM Import Job
**Responsibility:**
- Download OSM extracts for regions
- Filter relevant ways
- Insert into `curated_tracks`
- Default confidence: 3

**Run Frequency:** Weekly or on-demand

**Implementation:**
```rust
// backend/src/jobs/osm_import.rs
pub struct OsmImportJob {
    region: String,
    osm_pbf_path: String,
}
```

### Curated Tracks Import Job
**Responsibility:**
- Import GeoJSON/KML files
- Parse and validate geometry
- Assign confidence scores
- Mark verification date

**Run Frequency:** On-demand

### Confidence Update Job
**Responsibility:**
- Downgrade old unverified tracks
- Upgrade frequently used tracks
- Cross-validate with multiple sources

**Run Frequency:** Daily

**Logic:**
- Tracks > 1 year unverified: -1 confidence
- Tracks in multiple sources: +1 confidence
- Tracks used in many routes: +1 confidence

---

## 🚀 PoC Roadmap (3 Weeks)

### Week 1: Foundation
**Goals:** Working map with OSM tracks

**Tasks:**
1. ✅ Project setup (Docker, Git, structure)
2. ✅ Database schema + PostGIS setup
3. ✅ Backend skeleton (Axum, routes, models)
4. ✅ Frontend skeleton (Next.js, Mapbox)
5. Implement OSM import job
6. Basic track rendering
7. Layer toggle controls

**Deliverables:**
- Map showing OSM tracks
- Confidence-based coloring
- Layer toggles working

---

### Week 2: Core Features
**Goals:** Route creation and proposals

**Tasks:**
1. Route creation API
2. Route versioning system
3. Route editor component
4. Drawing tools integration
5. Proposal API
6. Proposal UI
7. Track info tooltips

**Deliverables:**
- Create and edit routes
- Submit proposals
- View route history

---

### Week 3: Polish & Data
**Goals:** Curated data and UX improvements

**Tasks:**
1. Curated tracks import job
2. Import sample rally data
3. Track snapping logic
4. Confidence update job
5. Region filtering
6. Performance optimization (clustering/vector tiles)
7. Mobile responsiveness

**Deliverables:**
- Real curated rally tracks
- Smooth UX on desktop/mobile
- Production-ready PoC

---

## 📦 Deployment Strategy

### PoC Deployment
**Option 1: Railway**
- Frontend: Railway static site
- Backend: Railway Rust service
- Database: Railway Postgres + PostGIS

**Option 2: Fly.io**
- Frontend: Fly.io app
- Backend: Fly.io app
- Database: Fly.io Postgres

**Option 3: VPS (DigitalOcean/Hetzner)**
- Docker Compose on single VPS
- Nginx reverse proxy
- Let's Encrypt SSL

### Production Considerations
- CDN for static assets (Cloudflare)
- Vector tiles for performance (MapTiler/Mapbox)
- Database backups
- Monitoring (Sentry, Datadog)
- Authentication (Clerk, Auth0)

---

## 🔮 Future Enhancements

### Phase 2: Advanced Features
- **ML Track Detection**: Satellite imagery analysis
- **Elevation Data**: SRTM integration
- **Weather Integration**: Historical weather patterns
- **Waypoint System**: Fuel, water, camping spots
- **Mobile App**: React Native / Flutter

### Phase 3: Community
- **User Accounts**: Authentication & profiles
- **Team Collaboration**: Shared routes
- **Comments & Discussions**: Per-route threads
- **Track Ratings**: Community feedback
- **Export Formats**: GPX, KML for GPS devices

### Phase 4: Intelligence
- **Route Optimization**: Shortest/safest path
- **Difficulty Scoring**: Terrain analysis
- **Time Estimation**: Based on surface/distance
- **Offline Mode**: Download regions
- **Live Tracking**: GPS integration during events

---

## 🧪 Testing Strategy

### Unit Tests
- Rust: `cargo test`
- Frontend: Jest + React Testing Library

### Integration Tests
- API endpoint tests
- Database query tests

### E2E Tests
- Playwright for critical flows
- Route creation flow
- Proposal submission flow

---

## 📚 Documentation Needs

### Developer Docs
- Setup guide (completed in README)
- API documentation (OpenAPI/Swagger)
- Database schema diagrams
- Data source integration guides

### User Docs
- User guide for route planning
- Confidence level explanation
- Proposal workflow
- Best practices

---

## 🎯 Success Metrics (PoC)

1. **Functionality**
   - [ ] Display 1000+ OSM tracks
   - [ ] Create and save routes
   - [ ] Submit proposals
   - [ ] Layer toggles work smoothly

2. **Performance**
   - [ ] Map loads in < 2s
   - [ ] Smooth panning/zooming
   - [ ] API response < 500ms

3. **Data Quality**
   - [ ] 50+ curated rally tracks
   - [ ] Correct confidence scoring
   - [ ] Accurate OSM filtering

4. **UX**
   - [ ] Intuitive interface
   - [ ] Mobile responsive
   - [ ] Clear confidence visualization

---

## 🛠️ Development Commands

```bash
# Database
docker-compose up -d              # Start PostgreSQL + PostGIS
docker-compose down               # Stop database

# Backend
cd backend
cargo build                       # Build
cargo run                         # Run server
cargo test                        # Run tests

# Frontend
cd frontend
npm install                       # Install dependencies
npm run dev                       # Development server
npm run build                     # Production build
npm run lint                      # Lint code

# Full setup
./scripts/setup.sh                # Automated setup
```

---

## 📞 Getting Help

### Resources
- **PostGIS Docs**: https://postgis.net/documentation/
- **Mapbox GL JS**: https://docs.mapbox.com/mapbox-gl-js/
- **Axum Docs**: https://docs.rs/axum/
- **Next.js Docs**: https://nextjs.org/docs

### Community
- OSM Community: https://www.openstreetmap.org/
- Overland routes: https://www.horizonsunlimited.com/

---

## ✅ Ready to Start!

This implementation plan provides everything needed to build the Dakar Planner PoC. The boilerplate is set up, the architecture is defined, and the roadmap is clear.

**Start with Week 1 tasks and iterate from there!**

Good luck! 🏆🏜️
