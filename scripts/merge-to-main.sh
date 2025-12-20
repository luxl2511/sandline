#!/bin/bash
set -e

echo "🔀 Merging to Main Branch"
echo "========================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get current branch
CURRENT_BRANCH=$(git branch --show-current)

echo -e "\n${YELLOW}Current branch: ${CURRENT_BRANCH}${NC}"

# Check if on feature branch
if [ "$CURRENT_BRANCH" = "main" ]; then
    echo -e "${GREEN}Already on main branch!${NC}"
    exit 0
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}❌ You have uncommitted changes. Please commit or stash them first.${NC}"
    git status
    exit 1
fi

echo -e "\n${YELLOW}Choose merge method:${NC}"
echo "1) Create Pull Request (recommended)"
echo "2) Direct merge to main"
echo "3) Fast-forward main to current branch"
echo "4) Cancel"
echo ""
read -p "Enter choice [1-4]: " choice

case $choice in
    1)
        echo -e "\n${YELLOW}Creating Pull Request...${NC}"

        # Check if gh is installed
        if ! command -v gh &> /dev/null; then
            echo -e "${YELLOW}GitHub CLI not installed. Opening browser...${NC}"
            REPO_URL=$(git config --get remote.origin.url | sed 's/\.git$//')
            echo "Visit: ${REPO_URL}/compare/main...${CURRENT_BRANCH}"
            exit 0
        fi

        # Create PR
        gh pr create \
            --base main \
            --head "$CURRENT_BRANCH" \
            --title "Merge ${CURRENT_BRANCH} to main" \
            --body "Automated merge of feature branch to main for deployment"

        echo -e "${GREEN}✅ Pull Request created!${NC}"
        echo "Review and merge at: $(gh pr view --web)"
        ;;

    2)
        echo -e "\n${YELLOW}Direct merge to main...${NC}"

        # Fetch latest
        git fetch origin

        # Switch to main
        git checkout main 2>/dev/null || git checkout -b main

        # Merge
        git merge "$CURRENT_BRANCH"

        # Push
        echo -e "\n${YELLOW}Pushing to remote...${NC}"
        git push origin main

        echo -e "${GREEN}✅ Merged to main successfully!${NC}"
        ;;

    3)
        echo -e "\n${RED}⚠️  This will force-update main to match your current branch.${NC}"
        read -p "Are you sure? (yes/no): " confirm

        if [ "$confirm" != "yes" ]; then
            echo "Cancelled."
            exit 0
        fi

        # Rename current branch to main
        git branch -M main

        # Force push
        git push -f origin main

        echo -e "${GREEN}✅ Fast-forwarded main to current branch!${NC}"
        ;;

    4)
        echo "Cancelled."
        exit 0
        ;;

    *)
        echo -e "${RED}Invalid choice.${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "1. Setup deployments (see DEPLOYMENT.md)"
echo "2. Configure GitHub Actions (add FLY_API_TOKEN)"
echo "3. Connect Vercel to your repository"
echo ""
echo "For detailed instructions: cat MERGE_TO_MAIN.md"
