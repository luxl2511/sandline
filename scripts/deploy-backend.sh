#!/bin/bash
set -e

echo "🚀 Deploying Backend to Fly.io"
echo "=============================="

cd backend

# Check if flyctl is installed
if ! command -v flyctl &>/dev/null; then
  echo "❌ flyctl is not installed. Install it from: https://fly.io/docs/hands-on/install-flyctl/"
  exit 1
fi

# Check if fly.toml exists
if [ ! -f fly.toml ]; then
  echo "❌ fly.toml not found. Run this script from the project root."
  exit 1
fi

echo "📦 Building Docker image..."
echo ""

echo "Preparing SQLX Statements"
cargo sqlx prepare
echo ""

# Deploy to Fly.io
echo "🚁 Deploying to Fly.io..."
flyctl deploy

echo ""
echo "✅ Backend deployed successfully!"
echo ""
echo "Next steps:"
echo "1. Set your DATABASE_URL secret:"
echo "   flyctl secrets set DATABASE_URL=postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres"
echo ""
echo "2. Get your app URL:"
echo "   flyctl info"
echo ""
echo "3. Update NEXT_PUBLIC_API_URL in Vercel with your Fly.io app URL"
