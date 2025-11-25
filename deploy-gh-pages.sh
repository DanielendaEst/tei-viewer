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

# Build with correct public URL for GitHub Pages
# DO NOT delete dist/ until we verify new build succeeds
echo "🔨 Building for GitHub Pages..."
echo "   Public URL: /$REPO_NAME/"
echo "   Note: Using dev build due to wasm-opt compatibility issues"
echo ""

# Build into a temporary directory first for safety
if [ -d "dist" ]; then
    echo "   (Keeping old dist/ until new build verified)"
fi

# Run build and capture exit code
trunk build --public-url /$REPO_NAME/
BUILD_EXIT_CODE=$?

# Check if build succeeded
if [ $BUILD_EXIT_CODE -ne 0 ]; then
    echo "❌ Error: Build failed with exit code $BUILD_EXIT_CODE"
    echo ""
    echo "Common issues:"
    echo "  - Make sure 'public/' directory exists (run ./sync_projects.sh first)"
    echo "  - Check that all Rust code compiles"
    echo "  - Verify Cargo.toml dependencies are correct"
    exit 1
fi

# Verify dist directory was created
if [ ! -d "dist" ]; then
    echo "❌ Error: Build succeeded but dist/ directory not created"
    echo "   This is unexpected. Try running: trunk build --public-url /$REPO_NAME/ manually"
    exit 1
fi

# Verify dist has content
if [ -z "$(ls -A dist 2>/dev/null)" ]; then
    echo "❌ Error: dist/ directory is empty"
    echo "   Build did not produce any output files"
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

# CRITICAL: Verify dist/ exists and has content BEFORE switching branches
if [ ! -d "dist" ]; then
    echo "❌ FATAL ERROR: dist/ directory does not exist!"
    echo "   Build succeeded but dist/ was not created."
    echo "   This is unexpected. Please check trunk output above."
    echo ""
    echo "⚠️  DEPLOYMENT ABORTED - Your projects/ folder is safe."
    exit 1
fi

if [ -z "$(ls -A dist 2>/dev/null)" ]; then
    echo "❌ FATAL ERROR: dist/ directory is empty!"
    echo "   Build did not produce any output files."
    echo ""
    echo "⚠️  DEPLOYMENT ABORTED - Your projects/ folder is safe."
    exit 1
fi

# Verify critical files exist in dist
if [ ! -f "dist/index.html" ]; then
    echo "❌ FATAL ERROR: dist/index.html not found!"
    echo "   Build incomplete."
    echo ""
    echo "⚠️  DEPLOYMENT ABORTED - Your projects/ folder is safe."
    exit 1
fi

echo "✅ dist/ directory verified - safe to proceed"
echo "   Contents: $(ls dist/ | tr '\n' ' ')"
echo ""
echo "📤 Deploying to GitHub Pages..."
echo ""

# Save the current commit hash and branch for reference
CURRENT_COMMIT=$(git rev-parse --short HEAD)
CURRENT_BRANCH=$(git branch --show-current)

echo "   Current branch: $CURRENT_BRANCH"
echo "   Will deploy from commit: $CURRENT_COMMIT"
echo ""

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
if ! cp -r dist/* . 2>/dev/null; then
    echo "❌ ERROR: Failed to copy dist/* to gh-pages branch"
    echo "   Aborting deployment and returning to $CURRENT_BRANCH"
    git checkout $CURRENT_BRANCH
    exit 1
fi
cp dist/.nojekyll . 2>/dev/null || true
echo "   Copied: $(ls | grep -v '^\.git$' | tr '\n' ' ')"

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

# Verify we're back on the correct branch
VERIFY_BRANCH=$(git branch --show-current)
if [ "$VERIFY_BRANCH" != "$CURRENT_BRANCH" ]; then
    echo "⚠️  WARNING: Expected to be on $CURRENT_BRANCH but on $VERIFY_BRANCH"
fi

# Clean up dist directory (optional - can keep for debugging)
# Uncomment the next line if you want to auto-delete dist/ after deployment
# rm -rf dist
echo "   Keeping dist/ for inspection (delete manually if needed: rm -rf dist)"

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
