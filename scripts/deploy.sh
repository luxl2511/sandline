#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Show usage
usage() {
  echo "Usage: ./scripts/deploy.sh [backend|frontend|all]"
  echo ""
  echo "Options:"
  echo "  backend   - Deploy backend to Fly.io"
  echo "  frontend  - Deploy frontend to Vercel"
  echo "  all       - Deploy both backend and frontend"
  echo ""
  echo "Examples:"
  echo "  ./scripts/deploy.sh backend"
  echo "  ./scripts/deploy.sh frontend"
  echo "  ./scripts/deploy.sh all"
  exit 1
}

# Deploy backend to Fly.io
deploy_backend() {
  echo -e "${BLUE}🚀 Deploying Backend to Fly.io${NC}"
  echo "=============================="
  echo ""

  cd backend

  # Check if flyctl is installed
  if ! command -v flyctl &>/dev/null; then
    echo -e "${RED}❌ flyctl is not installed${NC}"
    echo "Install: https://fly.io/docs/hands-on/install-flyctl/"
    exit 1
  fi

  # Check if fly.toml exists
  if [ ! -f fly.toml ]; then
    echo -e "${RED}❌ fly.toml not found${NC}"
    echo "Run this script from the project root."
    exit 1
  fi

  # Prepare SQLX statements
  echo "📦 Preparing SQLX statements..."
  cargo sqlx prepare
  echo ""

  # Deploy to Fly.io
  echo "🚁 Deploying to Fly.io..."
  flyctl deploy

  echo ""
  echo -e "${GREEN}✅ Backend deployed successfully!${NC}"
  echo ""

  cd ..
}

# Deploy frontend to Vercel
deploy_frontend() {
  echo -e "${BLUE}🚀 Deploying Frontend to Vercel${NC}"
  echo "==============================="
  echo ""

  cd frontend

  # Check if vercel is installed
  if ! command -v vercel &>/dev/null; then
    echo "Installing Vercel CLI..."
    npm install -g vercel
  fi

  # Deploy to Vercel production
  echo "🌐 Deploying to Vercel..."
  vercel --prod

  echo ""
  echo -e "${GREEN}✅ Frontend deployed successfully!${NC}"
  echo ""

  cd ..
}

# Main deployment logic
main() {
  if [ $# -eq 0 ]; then
    usage
  fi

  DEPLOY_TARGET=$1

  case $DEPLOY_TARGET in
    backend)
      deploy_backend
      ;;
    frontend)
      deploy_frontend
      ;;
    all)
      deploy_backend
      echo ""
      deploy_frontend
      echo ""
      echo -e "${GREEN}================================${NC}"
      echo -e "${GREEN}✅ Full Deployment Complete!${NC}"
      echo -e "${GREEN}================================${NC}"
      ;;
    *)
      echo -e "${RED}❌ Invalid option: $DEPLOY_TARGET${NC}"
      echo ""
      usage
      ;;
  esac
}

main "$@"
