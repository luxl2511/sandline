# Dakar Planner - Deployment Guide

This guide covers deploying the Dakar Planner application using:
- **Vercel** for the Next.js frontend
- **Fly.io** for the Rust backend
- **Supabase** for PostgreSQL + PostGIS database

---

## 🚨 Important: Main Branch Deployment

**Deployments are configured to run from the `main` branch.**

If you're currently on a feature branch (like `claude/dakar-planner-boilerplate-2zCQw`), you need to merge to `main` first:

```bash
# Quick merge to main
./scripts/merge-to-main.sh

# Or see detailed instructions
cat MERGE_TO_MAIN.md
```

**Automatic Deployments:**
- Push to `main` → GitHub Actions deploys backend to Fly.io
- Push to `main` → Vercel deploys frontend (after connecting repo)

---

## Prerequisites

1. **Accounts**
   - [Vercel Account](https://vercel.com/signup)
   - [Fly.io Account](https://fly.io/app/sign-up)
   - [Supabase Account](https://supabase.com/dashboard/sign-up)
   - [Mapbox Account](https://account.mapbox.com/auth/signup/)

2. **CLI Tools**
   ```bash
   # Install Vercel CLI
   npm install -g vercel

   # Install Fly.io CLI
   curl -L https://fly.io/install.sh | sh

   # Login to services
   vercel login
   flyctl auth login
   ```

---

## Step 1: Setup Supabase Database

### 1.1 Create Supabase Project

1. Go to [Supabase Dashboard](https://supabase.com/dashboard)
2. Click **"New Project"**
3. Fill in:
   - **Name**: `dakar-planner`
   - **Database Password**: (save this securely)
   - **Region**: Choose closest to your users
4. Click **"Create new project"**
5. Wait for database provisioning (~2 minutes)

### 1.2 Enable PostGIS Extension

1. In your project dashboard, go to **SQL Editor**
2. Copy and paste the entire contents of `scripts/setup-supabase.sql`
3. Click **"Run"**
4. Verify tables were created in **Table Editor**

### 1.3 Get Connection String

1. Go to **Project Settings** → **Database**
2. Under **Connection String**, copy the **URI** (not the transaction pooler)
3. It should look like:
   ```
   postgresql://postgres:[YOUR-PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres
   ```
4. Replace `[YOUR-PASSWORD]` with your actual database password
5. Save this for later

---

## Step 2: Deploy Backend to Fly.io

### 2.1 Initialize Fly.io App

```bash
cd backend

# Launch Fly.io app (creates fly.toml if not exists)
flyctl launch

# When prompted:
# - App name: dakar-planner-api (or your choice)
# - Region: Choose closest to your users
# - PostgreSQL database: No (we're using Supabase)
# - Redis: No
# - Deploy now: No
```

### 2.2 Set Environment Variables

```bash
# Set database URL secret
flyctl secrets set DATABASE_URL="postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres"

# Set CORS origins (update with your Vercel domain)
flyctl secrets set ALLOWED_ORIGINS="https://your-app.vercel.app,https://dakar-planner.vercel.app"

# Set Rust log level
flyctl secrets set RUST_LOG="info,dakar_planner_backend=debug"
```

### 2.3 Deploy

```bash
# Deploy to Fly.io
flyctl deploy

# Check status
flyctl status

# View logs
flyctl logs
```

### 2.4 Get Your Backend URL

```bash
flyctl info
```

Your backend URL will be: `https://your-app-name.fly.dev`

**Save this URL** - you'll need it for the frontend!

### 2.5 Test Backend

```bash
# Health check
curl https://your-app-name.fly.dev/api/tracks

# Should return JSON array (empty or with sample data)
```

---

## Step 3: Deploy Frontend to Vercel

### 3.1 Get Mapbox Token

1. Go to [Mapbox Account](https://account.mapbox.com/access-tokens/)
2. Click **"Create a token"**
3. Give it a name: `Dakar Planner Production`
4. Copy the token (starts with `pk.`)

### 3.2 Deploy via Vercel CLI

```bash
cd frontend

# Deploy
vercel --prod

# Follow prompts:
# - Set up and deploy: Yes
# - Which scope: Your account
# - Link to existing project: No
# - Project name: dakar-planner
# - Directory: ./
# - Override settings: No
```

### 3.3 Set Environment Variables

**Option A: Via Vercel Dashboard**
1. Go to your project in [Vercel Dashboard](https://vercel.com/dashboard)
2. Go to **Settings** → **Environment Variables**
3. Add:
   - `NEXT_PUBLIC_API_URL`: `https://your-app-name.fly.dev`
   - `NEXT_PUBLIC_MAPBOX_TOKEN`: `pk.your_mapbox_token`
4. Click **"Save"**
5. Trigger redeploy: **Deployments** → **...** → **Redeploy**

**Option B: Via CLI**
```bash
vercel env add NEXT_PUBLIC_API_URL production
# Paste: https://your-app-name.fly.dev

vercel env add NEXT_PUBLIC_MAPBOX_TOKEN production
# Paste: pk.your_mapbox_token

# Redeploy
vercel --prod
```

### 3.4 Get Your Frontend URL

Your app will be live at: `https://your-project-name.vercel.app`

### 3.5 Update CORS in Backend

Update the backend CORS to include your actual Vercel URL:

```bash
cd backend
flyctl secrets set ALLOWED_ORIGINS="https://your-project-name.vercel.app"
```

---

## Step 4: Verify Deployment

### 4.1 Test Full Stack

1. Visit your Vercel URL: `https://your-project-name.vercel.app`
2. You should see:
   - ✅ Map loads
   - ✅ Layer controls work
   - ✅ Sample tracks appear (if you ran setup-supabase.sql)

### 4.2 Check Browser Console

Open DevTools → Console. You should see:
- No CORS errors
- API calls to Fly.io succeeding
- Mapbox map loading

### 4.3 Test API Directly

```bash
# List tracks
curl https://your-app-name.fly.dev/api/tracks

# Should return JSON array
```

---

## Architecture Overview

```
┌─────────────────┐
│   Vercel        │
│   (Frontend)    │
│   Next.js App   │
└────────┬────────┘
         │ HTTPS
         │
┌────────▼────────┐
│   Fly.io        │
│   (Backend)     │
│   Rust API      │
└────────┬────────┘
         │ PostgreSQL
         │
┌────────▼────────┐
│   Supabase      │
│   (Database)    │
│   PostGIS       │
└─────────────────┘
```

---

## Environment Variables Summary

### Frontend (Vercel)
```env
NEXT_PUBLIC_API_URL=https://your-app-name.fly.dev
NEXT_PUBLIC_MAPBOX_TOKEN=pk.your_mapbox_token
```

### Backend (Fly.io)
```env
DATABASE_URL=postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres
ALLOWED_ORIGINS=https://your-project-name.vercel.app
RUST_LOG=info,dakar_planner_backend=debug
HOST=0.0.0.0
PORT=8080
```

---

## Deployment Scripts

We've included automated deployment scripts:

### Deploy Backend
```bash
./scripts/deploy-backend.sh
```

### Deploy Frontend
```bash
./scripts/deploy-frontend.sh
```

---

## Monitoring & Logs

### Vercel
```bash
# View logs
vercel logs

# View in dashboard
https://vercel.com/dashboard → Your Project → Logs
```

### Fly.io
```bash
# Stream logs
flyctl logs

# View metrics
flyctl dashboard
```

### Supabase
- Go to **Database** → **Logs**
- Monitor query performance
- Check table sizes

---

## Scaling

### Fly.io (Backend)

**Scale up instances:**
```bash
flyctl scale count 2  # Run 2 instances
```

**Scale up resources:**
```bash
flyctl scale vm shared-cpu-2x  # 2 CPUs
flyctl scale memory 512        # 512MB RAM
```

### Vercel (Frontend)
- Automatically scales
- No configuration needed
- Handles traffic spikes

### Supabase (Database)
- Free tier: 500MB storage, unlimited API requests
- Pro tier: Auto-scaling, point-in-time recovery
- Upgrade in dashboard if needed

---

## Cost Estimates

### Free Tier (PoC)
- **Vercel**: Free (hobby plan)
- **Fly.io**: ~$5/month (2 shared CPU VMs)
- **Supabase**: Free (up to 500MB)
- **Total**: ~$5/month

### Production (Low Traffic)
- **Vercel**: Free or $20/month (Pro)
- **Fly.io**: ~$20/month (dedicated CPU)
- **Supabase**: $25/month (Pro)
- **Total**: ~$45-65/month

---

## Troubleshooting

### CORS Errors

**Symptom:** Browser console shows CORS errors

**Fix:**
```bash
# Update backend CORS
cd backend
flyctl secrets set ALLOWED_ORIGINS="https://your-actual-vercel-url.vercel.app"
```

### Database Connection Errors

**Symptom:** Backend can't connect to Supabase

**Check:**
1. Verify DATABASE_URL is correct
2. Check Supabase project is running
3. Test connection:
   ```bash
   flyctl ssh console
   # Inside container:
   curl -I db.[PROJECT-REF].supabase.co:5432
   ```

### Map Not Loading

**Symptom:** Blank map or Mapbox error

**Check:**
1. NEXT_PUBLIC_MAPBOX_TOKEN is set correctly
2. Token is valid and not expired
3. Browser console for errors

### API Returns 404

**Symptom:** Frontend can't reach backend

**Check:**
1. NEXT_PUBLIC_API_URL matches Fly.io app URL
2. Backend is running: `flyctl status`
3. Test API directly: `curl https://your-app.fly.dev/api/tracks`

---

## Security Checklist

- [ ] Database password is strong and not committed to Git
- [ ] Fly.io secrets are set (not in fly.toml)
- [ ] CORS only allows your Vercel domain
- [ ] Mapbox token has URL restrictions enabled
- [ ] Supabase has Row Level Security configured (optional)
- [ ] Environment variables use production values

---

## Continuous Deployment

### Vercel (Automatic)
- Connects to your GitHub repo
- Auto-deploys on push to main branch
- Preview deployments for PRs

**Setup:**
1. Go to Vercel dashboard
2. Import Git Repository
3. Connect to your repo
4. Vercel handles the rest

### Fly.io (Manual or CI/CD)

**Manual:**
```bash
cd backend
flyctl deploy
```

**GitHub Actions** (add `.github/workflows/deploy.yml`):
```yaml
name: Deploy to Fly.io
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: superfly/flyctl-actions/setup-flyctl@master
      - run: flyctl deploy --remote-only
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
```

---

## Custom Domains

### Vercel
1. Go to **Settings** → **Domains**
2. Add your domain: `dakar-planner.com`
3. Update DNS records as instructed
4. SSL is automatic

### Fly.io
```bash
flyctl certs add api.dakar-planner.com
# Follow DNS instructions
```

Update frontend env:
```
NEXT_PUBLIC_API_URL=https://api.dakar-planner.com
```

---

## Backup & Recovery

### Supabase Backups
- Free tier: Daily backups (7 days retention)
- Pro tier: Point-in-time recovery

**Manual backup:**
```bash
pg_dump "postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres" > backup.sql
```

**Restore:**
```bash
psql "postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres" < backup.sql
```

---

## Next Steps

1. **Add Authentication**
   - Use Supabase Auth
   - Or integrate Clerk/Auth0

2. **Add Analytics**
   - Vercel Analytics (built-in)
   - PostHog or Plausible

3. **Set Up Monitoring**
   - Sentry for error tracking
   - Fly.io metrics dashboard

4. **Performance Optimization**
   - Add Redis for caching
   - Implement vector tiles for faster maps
   - CDN for static assets

---

## Support

- **Vercel Docs**: https://vercel.com/docs
- **Fly.io Docs**: https://fly.io/docs
- **Supabase Docs**: https://supabase.com/docs

---

**You're now live! 🎉**

Frontend: `https://your-project.vercel.app`
Backend: `https://your-app.fly.dev`
Database: Supabase
