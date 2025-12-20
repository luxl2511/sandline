# GitHub Actions Workflows

This directory contains automated CI/CD workflows for the Dakar Planner project.

## Workflows

### 🚀 Deployment Workflows

#### 1. `deploy-backend.yml` - Backend Deployment
**Triggers:** Push to `main` (when backend changes)

Automatically deploys the Rust backend to Fly.io.

**Required Secrets:**
- `FLY_API_TOKEN` - Your Fly.io API token

**Setup:**
```bash
# Get your Fly.io token
flyctl auth token

# Add to GitHub:
# Settings → Secrets and variables → Actions → New repository secret
# Name: FLY_API_TOKEN
# Value: [paste token]
```

---

#### 2. `deploy-frontend.yml` - Frontend Deployment
**Triggers:** Push to `main` (when frontend changes)

Automatically deploys the Next.js frontend to Vercel.

**Required Secrets:**
- `VERCEL_TOKEN` - Your Vercel API token
- `NEXT_PUBLIC_API_URL` - Backend API URL (e.g., `https://your-app.fly.dev`)
- `NEXT_PUBLIC_MAPBOX_TOKEN` - Mapbox access token

**Setup:**
```bash
# Get Vercel token
vercel tokens create

# Add to GitHub:
# - VERCEL_TOKEN
# - NEXT_PUBLIC_API_URL (https://your-app.fly.dev)
# - NEXT_PUBLIC_MAPBOX_TOKEN (pk.xxx)
```

**Alternative:** Use Vercel's GitHub integration instead (recommended):
1. Go to https://vercel.com/dashboard
2. Import GitHub repository
3. Vercel handles deployments automatically

---

#### 3. `deploy-all.yml` - Manual Full Deployment
**Triggers:** Manual workflow dispatch

Deploys both backend and frontend. Useful for coordinated releases.

**Usage:**
1. Go to Actions tab
2. Select "Deploy All"
3. Click "Run workflow"
4. Choose what to deploy

---

### ✅ CI Workflows

#### 4. `ci.yml` - Continuous Integration
**Triggers:** Push to `main` or Pull Requests

Validates code quality before deployment:
- **Backend:** Format check, linting, build, tests
- **Frontend:** Linting, type checking, build
- **Docker:** Validates Dockerfile builds

**No secrets required** - runs automatically.

---

## Quick Setup Guide

### Step 1: Add Fly.io Token

```bash
# Get token
flyctl auth token

# Add to GitHub
# Settings → Secrets → New secret
# Name: FLY_API_TOKEN
# Value: [token]
```

### Step 2: Choose Vercel Deployment Method

**Option A: Use Vercel GitHub Integration (Recommended)**
- Go to Vercel dashboard
- Import your GitHub repo
- Vercel auto-deploys on push to main
- ✅ No workflow needed

**Option B: Use GitHub Actions**
```bash
# Get Vercel token
vercel tokens create

# Add to GitHub Secrets:
# - VERCEL_TOKEN
# - NEXT_PUBLIC_API_URL
# - NEXT_PUBLIC_MAPBOX_TOKEN
```

### Step 3: Push to Main

```bash
git push origin main
```

🎉 **Auto-deployment is live!**

---

## Workflow Behavior

### On Push to `main`:

**If you change backend files:**
- ✅ `ci.yml` runs (validate)
- ✅ `deploy-backend.yml` runs (deploy to Fly.io)

**If you change frontend files:**
- ✅ `ci.yml` runs (validate)
- ✅ `deploy-frontend.yml` runs (deploy to Vercel)

**If you change both:**
- ✅ `ci.yml` runs (validate both)
- ✅ Both deployment workflows run

---

## Required GitHub Secrets

| Secret | Required For | How to Get |
|--------|--------------|------------|
| `FLY_API_TOKEN` | Backend deployment | `flyctl auth token` |
| `VERCEL_TOKEN` | Frontend deployment (if using Actions) | `vercel tokens create` |
| `NEXT_PUBLIC_API_URL` | Frontend build | Your Fly.io app URL |
| `NEXT_PUBLIC_MAPBOX_TOKEN` | Frontend build | Mapbox dashboard |
| `FLY_APP_NAME` | Health checks (optional) | Your Fly.io app name |

---

## Triggering Workflows Manually

**Via GitHub Web UI:**
1. Go to **Actions** tab
2. Select workflow
3. Click **"Run workflow"**
4. Choose branch and options

**Via GitHub CLI:**
```bash
# Deploy backend only
gh workflow run deploy-backend.yml

# Deploy frontend only
gh workflow run deploy-frontend.yml

# Deploy both
gh workflow run deploy-all.yml
```

---

## Monitoring Deployments

**View workflow status:**
```bash
gh run list
gh run view [run-id]
```

**View logs:**
- Go to Actions tab in GitHub
- Click on workflow run
- View individual job logs

**After deployment:**
```bash
# Check backend
curl https://your-app.fly.dev/api/tracks

# Check frontend
open https://your-app.vercel.app
```

---

## Troubleshooting

### Deployment fails with "FLY_API_TOKEN not found"
- Go to Settings → Secrets → Actions
- Verify `FLY_API_TOKEN` exists
- Token must have write permissions

### Vercel deployment fails
**Option 1:** Use Vercel GitHub integration instead
**Option 2:** Check all secrets are set correctly

### CI fails on pull requests
- CI runs checks but doesn't deploy
- Fix linting/type errors before merging

### Want to disable auto-deployment?
**Option 1:** Remove workflow file
```bash
git rm .github/workflows/deploy-backend.yml
git commit -m "Disable auto-deploy"
```

**Option 2:** Edit workflow to only run on `workflow_dispatch`
```yaml
on:
  workflow_dispatch:  # Manual only
```

---

## Best Practices

✅ **Always run CI before deploying**
- CI workflow validates code
- Catches errors before production

✅ **Use pull requests**
- CI runs on PRs automatically
- Review changes before merging

✅ **Monitor deployments**
- Check GitHub Actions tab
- Watch for failed deployments

✅ **Test after deployment**
- Visit your live URLs
- Check browser console for errors
- Verify API endpoints work

---

## Deployment Flow

```
Push to main
     ↓
CI Workflow runs
     ↓ (if passes)
Deploy Backend (if backend changed)
     ↓
Deploy Frontend (if frontend changed)
     ↓
Live! 🎉
```

---

Your CI/CD pipeline is ready! Push to `main` to deploy automatically. 🚀
