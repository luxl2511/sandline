# Dakar Planner - Data-Curated Route Planning

A collaborative route planning application for off-road and rally racing, featuring curated track data, confidence scoring, and collaborative proposals.

## Architecture

- **Frontend**: Next.js + TypeScript + Mapbox GL JS
- **Backend**: Rust + Axum
- **Database**: PostgreSQL + PostGIS

## Features

- 🗺️ **Curated Track Data**: OSM tracks, rally routes, and community-verified paths
- 🎯 **Confidence Scoring**: 1-5 scale for track reliability
- 🤝 **Collaborative Planning**: Route proposals and discussions
- 🎨 **Interactive Mapping**: Layer toggles, drawing tools, and track snapping
- 📊 **Data Sources**: OpenStreetMap, rally archives, satellite imagery

## Quick Start

### Local Development

**Prerequisites:**
- Docker & Docker Compose
- Node.js 18+
- Rust 1.70+

**Setup:**
```bash
# Automated setup
./scripts/setup.sh

# Or manual setup:
docker-compose up -d          # Start PostgreSQL + PostGIS
cd backend && cargo run       # Start backend
cd frontend && npm run dev    # Start frontend
```

Visit http://localhost:3000

### Production Deployment

Deploy to **Vercel** (frontend) + **Fly.io** (backend) + **Supabase** (database):

```bash
# See detailed deployment guide
cat DEPLOYMENT.md
```

**Quick Deploy:**
1. Create accounts on Vercel, Fly.io, and Supabase
2. Setup Supabase database: Run `scripts/setup-supabase.sql`
3. Deploy backend: `./scripts/deploy-backend.sh`
4. Deploy frontend: `./scripts/deploy-frontend.sh`

See [DEPLOYMENT.md](./DEPLOYMENT.md) for complete step-by-step instructions.

## Project Structure

```
├── frontend/          # Next.js application
├── backend/           # Rust API server
├── scripts/           # Setup and deployment scripts
└── docker-compose.yml # PostgreSQL + PostGIS
```

## Data Model

- **curated_tracks**: Core track database with confidence scores
- **routes**: User-created routes
- **route_versions**: Version history
- **route_proposals**: Collaborative proposals

## Documentation

- **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** - 3-week PoC development roadmap
- **[DEPLOYMENT.md](./DEPLOYMENT.md)** - Production deployment guide (Vercel + Fly.io + Supabase)

## Tech Stack

**Frontend:**
- Next.js 14 (App Router) + TypeScript
- Mapbox GL JS for interactive maps
- Tailwind CSS for styling
- Zustand for state management

**Backend:**
- Rust + Axum web framework
- SQLx for database queries
- PostGIS for spatial operations

**Database:**
- PostgreSQL 15 + PostGIS 3.4
- Supabase for production hosting

**Deployment:**
- Vercel (frontend)
- Fly.io (backend)
- Supabase (database)

## License

MIT
