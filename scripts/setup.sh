#!/bin/bash
set -e

echo "🏗️  Dakar Planner - Setup Script"
echo "================================"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"
command -v docker >/dev/null 2>&1 || { echo "❌ Docker is required but not installed."; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "❌ Docker Compose is required but not installed."; exit 1; }
command -v node >/dev/null 2>&1 || { echo "❌ Node.js is required but not installed."; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo is required but not installed."; exit 1; }
echo -e "${GREEN}✓ All prerequisites found${NC}"

# Setup database
echo -e "\n${YELLOW}Starting PostgreSQL + PostGIS...${NC}"
docker-compose up -d
echo "Waiting for database to be ready..."
sleep 5
echo -e "${GREEN}✓ Database is running${NC}"

# Setup backend
echo -e "\n${YELLOW}Setting up backend...${NC}"
cd backend

if [ ! -f .env ]; then
    echo "Creating .env file..."
    cp .env.example .env
fi

echo "Building backend..."
cargo build
echo -e "${GREEN}✓ Backend setup complete${NC}"

cd ..

# Setup frontend
echo -e "\n${YELLOW}Setting up frontend...${NC}"
cd frontend

if [ ! -f .env.local ]; then
    echo "Creating .env.local file..."
    cp .env.example .env.local
    echo ""
    echo -e "${YELLOW}⚠️  IMPORTANT: Please add your Mapbox token to frontend/.env.local${NC}"
    echo "   Get your token at: https://account.mapbox.com/access-tokens/"
fi

echo "Installing frontend dependencies..."
npm install
echo -e "${GREEN}✓ Frontend setup complete${NC}"

cd ..

# Summary
echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}✅ Setup Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo "Next steps:"
echo "1. Add your Mapbox token to frontend/.env.local"
echo "2. Start the backend:"
echo "   cd backend && cargo run"
echo "3. Start the frontend (in new terminal):"
echo "   cd frontend && npm run dev"
echo "4. Visit http://localhost:3000"
echo ""
echo "Database: postgresql://dakar_user:dakar_pass_dev_only@localhost:5432/dakar_planner"
