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

### Prerequisites

- Docker & Docker Compose
- Node.js 18+
- Rust 1.70+
- PostgreSQL 15+ with PostGIS

### Setup

```bash
# Clone and enter directory
cd sandline

# Start database
docker-compose up -d

# Setup backend
cd backend
cp .env.example .env
cargo build
cargo run --bin migrate
cargo run

# Setup frontend (in new terminal)
cd frontend
cp .env.example .env.local
npm install
npm run dev
```

Visit http://localhost:3000

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

## Development Roadmap

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for detailed PoC roadmap.

## License

MIT
