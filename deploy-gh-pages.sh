#!/bin/bash

# TEI Viewer - Automated GitHub Pages Deployment Script
# This script builds and deploys the TEI Viewer to GitHub Pages

set -e

echo "🚀 TEI Viewer - GitHub Pages Deployment"
echo "=========================================="
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not a git repository"
    echo "   Please run this script from the tei-viewer directory"
    exit 1
fi

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ Error: trunk is not installed"
    echo ""
    echo "Please install trunk with:"
    echo "  cargo install trunk"
    echo ""
    exit 1
fi

# Get the repository name from git remote
REPO_NAME=$(basename -s .git `git config --get remote.origin.url` 2>/dev/null || echo "tei-viewer")
CURRENT_BRANCH=$(git branch --show-current)

echo "📊 Repository: $REPO_NAME"
echo "🌿 Current branch: $CURRENT_BRANCH"
echo ""

# Check for uncommitted changes
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "⚠️  Warning: You have uncommitted changes"
    read -p "   Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Deployment cancelled."
        exit 1
    fi
fi

# Sync projects from tei-viewer/projects to public/projects
echo "📦 Syncing projects..."
if [ -f "sync_projects.sh" ]; then
    ./sync_projects.sh
else
    echo "⚠️  Warning: sync_projects.sh not found"
    echo "   Falling back to manual copy..."

    # Ensure public directory exists with projects
    if [ ! -d "public/projects" ]; then
        mkdir -p public
        if [ -d "projects" ]; then
            echo "📁 Copying projects to public directory..."
            cp -r projects public/
        else
            echo "⚠️  Warning: projects/ directory not found"
        fi
    fi
fi
echo ""

# Clean previous build
if [ -d "dist" ]; then
    echo "🧹 Cleaning previous build..."
    rm -rf dist
fi

# Build with correct public URL for GitHub Pages
echo "🔨 Building for GitHub Pages..."
echo "   Public URL: /$REPO_NAME/"
echo "   Note: Using dev build due to wasm-opt compatibility issues"
echo ""

trunk build --public-url /$REPO_NAME/

if [ ! -d "dist" ]; then
    echo "❌ Error: Build failed - dist directory not created"
    exit 1
fi

# Add .nojekyll file to disable Jekyll processing
echo "📝 Adding .nojekyll file..."
touch dist/.nojekyll

# Optional: Add CNAME file for custom domain
# Uncomment and modify the following lines if you have a custom domain:
# echo "www.yourdomain.com" > dist/CNAME
# echo "✅ Added CNAME file for custom domain"

echo ""
echo "✅ Build complete!"
echo ""
echo "📤 Deploying to GitHub Pages..."
echo ""

# Save the current commit hash for reference
CURRENT_COMMIT=$(git rev-parse --short HEAD)

# Stash any changes if needed
STASH_NEEDED=false
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "💾 Stashing local changes..."
    git stash push -m "Auto-stash before gh-pages deployment"
    STASH_NEEDED=true
fi

# Switch to gh-pages branch (create if it doesn't exist)
echo "🔀 Switching to gh-pages branch..."
if git show-ref --verify --quiet refs/heads/gh-pages; then
    git checkout gh-pages
else
    git checkout --orphan gh-pages
fi

# Remove all existing files (except .git)
echo "🗑️  Clearing old deployment..."
git rm -rf . 2>/dev/null || true
find . -maxdepth 1 ! -name '.git' ! -name '.' ! -name '..' -exec rm -rf {} + 2>/dev/null || true

# Copy built files to root
echo "📋 Copying new build..."
cp -r dist/* .
cp dist/.nojekyll . 2>/dev/null || true

# Add all files
git add -A

# Commit
echo "💾 Committing deployment..."
TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")
git commit -m "Deploy to GitHub Pages - $TIMESTAMP (from $CURRENT_COMMIT)" || {
    echo "⚠️  No changes to deploy"
    git checkout $CURRENT_BRANCH
    if [ "$STASH_NEEDED" = true ]; then
        git stash pop
    fi
    exit 0
}

# Push to GitHub
echo "⬆️  Pushing to GitHub..."
git push origin gh-pages --force

# Return to original branch
echo "🔙 Returning to $CURRENT_BRANCH branch..."
git checkout $CURRENT_BRANCH

# Restore stashed changes if any
if [ "$STASH_NEEDED" = true ]; then
    echo "📦 Restoring stashed changes..."
    git stash pop
fi

echo ""
echo "=========================================="
echo "✅ Deployment successful!"
echo ""
echo "🌐 Your site will be available at:"
echo "   https://$(git config --get user.name 2>/dev/null || echo "yourusername").github.io/$REPO_NAME/"
echo ""
echo "📝 Next steps:"
echo "   1. Go to GitHub repository Settings → Pages"
echo "   2. Under 'Source', select branch 'gh-pages' and folder '/ (root)'"
echo "   3. Click 'Save'"
echo "   4. Wait 1-2 minutes for deployment to complete"
echo ""
echo "💡 Tip: You can check deployment status in the Actions tab"
echo ""
echo "=========================================="
