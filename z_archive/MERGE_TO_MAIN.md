# Merging to Main Branch for Deployment

This guide helps you merge your feature branch to `main` and set up automatic deployments.

## Current Setup

- **Feature Branch**: `claude/dakar-planner-boilerplate-2zCQw`
- **Deployment Branch**: `main`
- **CI/CD**: GitHub Actions deploys from `main` automatically

---

## Option 1: Merge via Pull Request (Recommended)

This is the recommended approach for production workflows.

### Step 1: Create Pull Request

```bash
# Make sure you're on your feature branch
git checkout claude/dakar-planner-boilerplate-2zCQw

# Push your latest changes
git push origin claude/dakar-planner-boilerplate-2zCQw

# Go to GitHub and create a PR
# https://github.com/YOUR_USERNAME/sandline/compare/main...claude/dakar-planner-boilerplate-2zCQw
```

Or use GitHub CLI:
```bash
gh pr create \
  --base main \
  --head claude/dakar-planner-boilerplate-2zCQw \
  --title "Add Dakar Planner boilerplate with deployment configs" \
  --body "Complete boilerplate for Dakar Planner with Vercel + Fly.io + Supabase deployment"
```

### Step 2: Review and Merge

1. Review the PR on GitHub
2. Click "Merge pull request"
3. Choose merge strategy:
   - **Squash and merge** (recommended for clean history)
   - **Create a merge commit** (preserves all commits)
   - **Rebase and merge** (linear history)

### Step 3: Deploy

Once merged to `main`, deployments will happen automatically:
- Backend: GitHub Actions deploys to Fly.io
- Frontend: Vercel deploys automatically (after connecting repo)

---

## Option 2: Direct Merge (Quick)

If you don't need PR review, merge directly:

```bash
# Fetch latest from remote
git fetch origin

# Switch to main branch (or create it)
git checkout -b main 2>/dev/null || git checkout main

# Merge feature branch
git merge claude/dakar-planner-boilerplate-2zCQw

# Push to remote
git push origin main
```

---

## Option 3: Fast-Forward Merge (If main is empty)

If `main` branch doesn't exist or is behind:

```bash
# Rename current branch to main
git checkout claude/dakar-planner-boilerplate-2zCQw
git branch -m main

# Force push to create/update main
git push -f origin main
```

⚠️ **Warning**: Only use `-f` if you're sure main doesn't have important changes!

---

## Post-Merge: Setup Deployments

### 1. Setup Backend Deployment (Fly.io)

```bash
# Deploy backend first
cd backend
flyctl launch --no-deploy

# Set secrets
flyctl secrets set DATABASE_URL="postgresql://postgres:PASSWORD@db.PROJECT-REF.supabase.co:5432/postgres"
flyctl secrets set ALLOWED_ORIGINS="https://your-app.vercel.app"

# Deploy
flyctl deploy
```

### 2. Setup GitHub Actions

Add Fly.io token to GitHub Secrets:

```bash
# Get your Fly.io token
flyctl auth token

# Add to GitHub:
# 1. Go to: Settings → Secrets and variables → Actions
# 2. Click "New repository secret"
# 3. Name: FLY_API_TOKEN
# 4. Value: [your token]
```

Now every push to `main` auto-deploys backend!

### 3. Setup Frontend Deployment (Vercel)

**Connect GitHub to Vercel:**

1. Go to [Vercel Dashboard](https://vercel.com/dashboard)
2. Click "Add New" → "Project"
3. Import your GitHub repo
4. Configure:
   - **Framework**: Next.js
   - **Root Directory**: `frontend`
   - **Build Command**: `npm run build`
   - **Output Directory**: `.next`

5. Add environment variables:
   - `NEXT_PUBLIC_API_URL`: `https://your-app.fly.dev`
   - `NEXT_PUBLIC_MAPBOX_TOKEN`: `pk.your_mapbox_token`

6. Deploy!

Now every push to `main` auto-deploys frontend!

---

## Deployment Workflow

After setup, your workflow is:

```bash
# 1. Make changes
git checkout -b feature/my-new-feature
# ... make changes ...

# 2. Commit and push
git add .
git commit -m "Add new feature"
git push origin feature/my-new-feature

# 3. Create PR and merge to main
gh pr create --base main

# 4. Auto-deployment kicks in!
# - GitHub Actions deploys backend to Fly.io
# - Vercel deploys frontend
```

---

## Branch Strategy

**Recommended setup:**

```
main                    ← Production branch (deploys automatically)
  ├── develop          ← Development branch (optional)
  └── feature/*        ← Feature branches
```

**Protection rules for main:**
- Require PR reviews
- Require status checks to pass
- No direct pushes (except initial setup)

---

## Verification Checklist

After merging to main:

- [ ] GitHub Actions workflow runs successfully
- [ ] Backend deploys to Fly.io (check logs: `flyctl logs`)
- [ ] Frontend deploys to Vercel (check Vercel dashboard)
- [ ] Environment variables are set correctly
- [ ] API is accessible: `curl https://your-app.fly.dev/api/tracks`
- [ ] Frontend loads: `https://your-app.vercel.app`
- [ ] No CORS errors in browser console
- [ ] Map loads with Mapbox token

---

## Troubleshooting

### GitHub Actions fails

```bash
# Check workflow status
gh run list

# View logs
gh run view [run-id]

# Common issues:
# - Missing FLY_API_TOKEN secret
# - Workflow file syntax error
# - Insufficient permissions
```

### Backend deployment fails

```bash
# Check Fly.io status
flyctl status

# View logs
flyctl logs

# SSH into container
flyctl ssh console

# Common issues:
# - Missing DATABASE_URL secret
# - Database connection refused
# - Port binding issues
```

### Frontend deployment fails

```bash
# Check Vercel logs
vercel logs

# Common issues:
# - Missing environment variables
# - Build errors (run `npm run build` locally first)
# - API connection issues
```

---

## Rolling Back

If deployment breaks something:

### Rollback Frontend (Vercel)
1. Go to Vercel dashboard → Deployments
2. Find previous working deployment
3. Click "..." → "Promote to Production"

### Rollback Backend (Fly.io)
```bash
# List recent releases
flyctl releases

# Rollback to previous version
flyctl releases rollback [version]
```

### Rollback Git
```bash
# Revert last commit on main
git checkout main
git revert HEAD
git push origin main
```

---

## Next Steps

1. **Merge your feature branch to main** (choose option above)
2. **Setup Fly.io deployment** (see DEPLOYMENT.md)
3. **Setup Vercel deployment** (see DEPLOYMENT.md)
4. **Configure GitHub Actions** (add FLY_API_TOKEN)
5. **Test your deployment** (verify everything works)
6. **Set up branch protection** (optional, for team workflows)

---

**Ready to deploy! 🚀**

For detailed deployment instructions, see [DEPLOYMENT.md](./DEPLOYMENT.md)
