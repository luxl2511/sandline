#!/bin/bash
set -e

echo "🚀 Dakar Planner - Quick Deployment"
echo "===================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${RED}❌ You must be on the 'main' branch to deploy.${NC}"
    echo -e "${YELLOW}Current branch: ${CURRENT_BRANCH}${NC}"
    echo ""
    echo "Run: ./scripts/merge-to-main.sh"
    exit 1
fi

echo -e "${GREEN}✓ On main branch${NC}"
echo ""

# Step 1: Supabase
echo -e "${BLUE}Step 1: Supabase Database${NC}"
echo "=========================="
echo ""
echo "1. Go to https://supabase.com/dashboard"
echo "2. Create new project: 'dakar-planner'"
echo "3. Save your database password"
echo "4. Open SQL Editor"
echo "5. Run: scripts/setup-supabase.sql"
echo "6. Get connection string from Settings → Database"
echo ""
read -p "Press Enter when Supabase is ready..."
echo ""
read -p "Enter your Supabase DATABASE_URL: " DATABASE_URL

if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}❌ DATABASE_URL is required${NC}"
    exit 1
fi

# Step 2: Fly.io
echo ""
echo -e "${BLUE}Step 2: Fly.io Backend${NC}"
echo "======================"
echo ""

cd backend

# Check if flyctl is installed
if ! command -v flyctl &> /dev/null; then
    echo -e "${RED}❌ flyctl is not installed${NC}"
    echo "Install: curl -L https://fly.io/install.sh | sh"
    exit 1
fi

# Check if already launched
if [ ! -f fly.toml ]; then
    echo "Launching new Fly.io app..."
    flyctl launch --no-deploy
else
    echo -e "${GREEN}✓ Fly.io app already configured${NC}"
fi

# Set secrets
echo "Setting Fly.io secrets..."
flyctl secrets set DATABASE_URL="$DATABASE_URL"

read -p "Enter your Vercel frontend URL (or press Enter to skip): " VERCEL_URL
if [ ! -z "$VERCEL_URL" ]; then
    flyctl secrets set ALLOWED_ORIGINS="http://localhost:3000,${VERCEL_URL}"
else
    flyctl secrets set ALLOWED_ORIGINS="http://localhost:3000"
fi

# Deploy backend
echo ""
echo "Deploying backend to Fly.io..."
flyctl deploy

# Get backend URL
BACKEND_URL=$(flyctl info --json | grep -o '"Hostname":"[^"]*' | cut -d'"' -f4)
BACKEND_URL="https://${BACKEND_URL}"

echo ""
echo -e "${GREEN}✅ Backend deployed!${NC}"
echo -e "URL: ${BACKEND_URL}"

cd ..

# Step 3: Vercel
echo ""
echo -e "${BLUE}Step 3: Vercel Frontend${NC}"
echo "======================="
echo ""

cd frontend

# Check if vercel is installed
if ! command -v vercel &> /dev/null; then
    echo "Installing Vercel CLI..."
    npm install -g vercel
fi

# Get Mapbox token
echo ""
read -p "Enter your Mapbox access token (pk.xxx): " MAPBOX_TOKEN

if [ -z "$MAPBOX_TOKEN" ]; then
    echo -e "${RED}❌ Mapbox token is required${NC}"
    echo "Get one at: https://account.mapbox.com/access-tokens/"
    exit 1
fi

# Set environment variables
echo ""
echo "Setting environment variables..."
echo "$BACKEND_URL" | vercel env add NEXT_PUBLIC_API_URL production
echo "$MAPBOX_TOKEN" | vercel env add NEXT_PUBLIC_MAPBOX_TOKEN production

# Deploy frontend
echo ""
echo "Deploying frontend to Vercel..."
vercel --prod

cd ..

# Summary
echo ""
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}✅ Deployment Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo -e "${YELLOW}Backend:${NC} ${BACKEND_URL}"
echo -e "${YELLOW}Frontend:${NC} Check Vercel dashboard for URL"
echo ""
echo "Next steps:"
echo "1. Visit your frontend URL"
echo "2. Check that the map loads"
echo "3. Verify API connection in browser console"
echo ""
echo "If you set a Vercel URL, update CORS:"
echo "  flyctl secrets set ALLOWED_ORIGINS=\"http://localhost:3000,https://your-app.vercel.app\""
echo ""
echo "View logs:"
echo "  Backend:  flyctl logs"
echo "  Frontend: vercel logs"
