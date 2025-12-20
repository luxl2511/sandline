# GitHub Actions Workflows

## Setup Instructions

### Backend Deployment (Fly.io)

To enable automatic backend deployment to Fly.io on push to main:

1. **Get your Fly.io API token:**
   ```bash
   flyctl auth token
   ```

2. **Add token to GitHub Secrets:**
   - Go to your repo → Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `FLY_API_TOKEN`
   - Value: Your Fly.io API token
   - Click "Add secret"

3. **Push to main branch:**
   ```bash
   git push origin main
   ```

Now, any changes to the `backend/` directory will trigger automatic deployment!

### Frontend Deployment (Vercel)

Vercel handles this automatically when you connect your GitHub repository:

1. Go to [Vercel Dashboard](https://vercel.com/dashboard)
2. Click "Add New" → "Project"
3. Import your GitHub repository
4. Vercel will auto-deploy on every push to main

No workflow needed - Vercel's GitHub integration handles it!

## Manual Deployment

You can also trigger workflows manually:
- Go to Actions tab → Select workflow → Run workflow
