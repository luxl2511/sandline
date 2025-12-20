#!/bin/bash
set -e

echo "🚀 Deploying Frontend to Vercel"
echo "================================"

# Check if vercel CLI is installed
if ! command -v vercel &> /dev/null; then
    echo "Installing Vercel CLI..."
    npm install -g vercel
fi

cd frontend

echo "📦 Building and deploying to Vercel..."
echo ""

# Deploy to Vercel
vercel --prod

echo ""
echo "✅ Frontend deployed successfully!"
echo ""
echo "Next steps:"
echo "1. Set environment variables in Vercel dashboard:"
echo "   - NEXT_PUBLIC_API_URL: Your Fly.io backend URL"
echo "   - NEXT_PUBLIC_MAPBOX_TOKEN: Your Mapbox access token"
echo ""
echo "2. Or set them via CLI:"
echo "   vercel env add NEXT_PUBLIC_API_URL production"
echo "   vercel env add NEXT_PUBLIC_MAPBOX_TOKEN production"
