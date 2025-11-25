# GitHub Pages Deployment Guide for TEI Viewer

## What is GitHub Pages?

**GitHub Pages** is a free static website hosting service provided by GitHub. It takes HTML, CSS, and JavaScript files directly from a repository on GitHub and publishes them as a website.

### Key Features:
- ✅ **100% Free** (for public repositories)
- ✅ **Automatic HTTPS** (free SSL certificate)
- ✅ **Custom domains** supported
- ✅ **CDN-backed** (fast global delivery)
- ✅ **Easy deployment** (git-based)
- ✅ **Perfect for static sites** (like this TEI Viewer!)

### Limitations:
- ❌ 1 GB repository size limit
- ❌ 100 GB bandwidth per month (soft limit)
- ❌ 10 builds per hour
- ❌ Static sites only (no server-side code)
- ❌ Public repos only for free accounts (private repos need Pro)

---

## How GitHub Pages Works

```
Your Repository
    └── gh-pages branch (or main branch /docs folder)
        └── HTML/CSS/JS files
            ↓
    GitHub Pages automatically builds and deploys
            ↓
    Your site is live at:
    https://username.github.io/repository-name/
```

### Three Hosting Options:

1. **From a branch** (e.g., `gh-pages` branch) ← Most common
2. **From `/docs` folder** on main branch
3. **From root of main branch** (for simple sites)

---

## Deploying TEI Viewer to GitHub Pages

### Option 1: Manual Deployment (Easiest to Understand)

#### Step 1: Build Your Site
```bash
# From your tei-viewer directory
./deploy.sh

# This creates the dist/ folder with your built site
```

#### Step 2: Create and Push to gh-pages Branch
```bash
# Create a new orphan branch (no history)
git checkout --orphan gh-pages

# Remove all files from staging
git rm -rf .

# Copy built files to root
cp -r dist/* .
cp dist/.* . 2>/dev/null || true  # Copy hidden files if any

# Add and commit
git add .
git commit -m "Deploy TEI Viewer to GitHub Pages"

# Push to GitHub
git push origin gh-pages --force

# Go back to main branch
git checkout main
```

#### Step 3: Enable GitHub Pages
1. Go to your repository on GitHub
2. Click **Settings** → **Pages** (left sidebar)
3. Under "Source", select:
   - Branch: `gh-pages`
   - Folder: `/ (root)`
4. Click **Save**
5. Wait 1-2 minutes for deployment

Your site will be live at: `https://yourusername.github.io/tei-viewer/`

---

### Option 2: Automated Deployment with GitHub Actions (Recommended)

This automatically rebuilds and deploys whenever you push to main.

#### Step 1: Create GitHub Actions Workflow

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [ main ]
  workflow_dispatch:  # Allow manual trigger

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install Trunk
        run: cargo install trunk

      - name: Setup public directory
        run: |
          mkdir -p public
          if [ -d "projects" ]; then
            cp -r projects public/
          fi

      - name: Build with Trunk
        run: trunk build --release --public-url /${{ github.event.repository.name }}/

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v2
        with:
          path: ./dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v3
```

#### Step 2: Enable GitHub Pages
1. Go to repository **Settings** → **Pages**
2. Under "Source", select: **GitHub Actions**
3. Commit and push the workflow file
4. GitHub will automatically build and deploy!

#### Step 3: Push and Watch
```bash
git add .github/workflows/deploy.yml
git commit -m "Add GitHub Pages deployment workflow"
git push origin main

# Go to the "Actions" tab to watch the deployment
```

---

### Option 3: Using gh-pages npm Package (Alternative)

#### Install the package:
```bash
npm install -g gh-pages
```

#### Create a deployment script:
```bash
#!/bin/bash
# deploy-gh-pages.sh

# Build
./deploy.sh

# Deploy
gh-pages -d dist -b gh-pages
```

#### Deploy:
```bash
chmod +x deploy-gh-pages.sh
./deploy-gh-pages.sh
```

---

## Important Configuration for GitHub Pages

### Base URL Issue

GitHub Pages serves your site at `https://username.github.io/repo-name/`, not at the root.

**Problem:** Your app might load assets from `/image.png` instead of `/repo-name/image.png`

**Solution:** Configure the public URL when building:

```bash
# Build with correct base path
trunk build --release --public-url /tei-viewer/
```

Or in your Trunk.toml:
```toml
[build]
public_url = "/tei-viewer/"
```

### Update Workflow (if using Actions)

The workflow above already includes this:
```yaml
trunk build --release --public-url /${{ github.event.repository.name }}/
```

---

## Custom Domain Setup

### Step 1: Configure DNS (at your domain registrar)

Add a CNAME record:
```
Type: CNAME
Host: www (or @)
Value: yourusername.github.io
```

Or for apex domain, add A records:
```
Type: A
Host: @
Value: 185.199.108.153
Value: 185.199.109.153
Value: 185.199.110.153
Value: 185.199.111.153
```

### Step 2: Configure GitHub

1. Go to **Settings** → **Pages**
2. Under "Custom domain", enter: `www.yourdomain.com`
3. Check "Enforce HTTPS" (after DNS propagates)
4. Create a `CNAME` file in your `dist/` folder:
   ```
   www.yourdomain.com
   ```

### Step 3: Update Build Script

Make sure the CNAME file is included:
```bash
# In deploy.sh, after trunk build:
echo "www.yourdomain.com" > dist/CNAME
```

---

## Complete Deployment Example

Here's a complete script for manual deployment:

```bash
#!/bin/bash
# deploy-to-gh-pages.sh

set -e

echo "🔨 Building TEI Viewer for GitHub Pages..."

# Get the repository name from git
REPO_NAME=$(basename -s .git `git config --get remote.origin.url`)

# Build with correct public URL
echo "📦 Building for public URL: /$REPO_NAME/"
trunk build --release --public-url /$REPO_NAME/

echo "📋 Preparing deployment..."

# Save current branch
CURRENT_BRANCH=$(git branch --show-current)

# Create/switch to gh-pages branch
git checkout gh-pages 2>/dev/null || git checkout --orphan gh-pages

# Remove old files (but keep .git)
git rm -rf . 2>/dev/null || true
rm -rf * 2>/dev/null || true

# Copy built files
cp -r dist/* .

# Optional: Add CNAME for custom domain
# echo "www.yourdomain.com" > CNAME

# Commit and push
git add .
git commit -m "Deploy to GitHub Pages - $(date)"
git push origin gh-pages --force

# Return to original branch
git checkout $CURRENT_BRANCH

echo "✅ Deployment complete!"
echo "🌐 Your site will be available at:"
echo "   https://yourusername.github.io/$REPO_NAME/"
echo ""
echo "⏱️  It may take 1-2 minutes to go live."
```

Save as `deploy-gh-pages.sh` and run:
```bash
chmod +x deploy-gh-pages.sh
./deploy-gh-pages.sh
```

---

## Troubleshooting

### Issue: 404 Page Not Found

**Cause:** Files not in the right location or wrong branch selected

**Solution:**
1. Check the `gh-pages` branch has files in the root
2. Verify Settings → Pages shows the correct branch
3. Make sure `index.html` exists in the root

### Issue: Blank Page / Assets Not Loading

**Cause:** Wrong base URL

**Solution:**
Build with correct public URL:
```bash
trunk build --release --public-url /your-repo-name/
```

### Issue: "Published site is having problems"

**Cause:** Build errors or missing index.html

**Solution:**
1. Check the `gh-pages` branch has `index.html`
2. Check browser console for errors
3. Verify WASM files are present

### Issue: Changes Not Appearing

**Cause:** GitHub caching or deployment delay

**Solution:**
1. Wait 2-3 minutes after pushing
2. Hard refresh: Ctrl+F5 (Windows) or Cmd+Shift+R (Mac)
3. Check the Actions tab for build status (if using Actions)

### Issue: WASM Not Loading

**Cause:** GitHub Pages might not set correct MIME type (rare)

**Solution:**
This is usually automatic, but verify in browser DevTools that `.wasm` files have `Content-Type: application/wasm`

---

## Comparison: GitHub Pages vs Other Options

| Feature | GitHub Pages | Netlify | Vercel |
|---------|-------------|---------|--------|
| **Price** | Free | Free tier | Free tier |
| **Setup** | Medium | Easy | Easy |
| **Build Time** | Slower | Fast | Fast |
| **Bandwidth** | 100GB/mo | 100GB/mo | 100GB/mo |
| **Custom Domain** | Yes | Yes | Yes |
| **HTTPS** | Auto | Auto | Auto |
| **Build Minutes** | Unlimited* | 300/mo | 6000/mo |
| **Deployment** | Git-based | Git/CLI/Drop | Git/CLI |
| **Best For** | GitHub repos | Any project | Any project |

*10 builds per hour limit

---

## Best Practices

### 1. Use GitHub Actions for Automation
- Automatic deployment on every push
- No manual steps required
- Consistent builds

### 2. Keep gh-pages Branch Clean
- Only built files, no source code
- Use automation to update it
- Never edit directly

### 3. Test Locally First
```bash
./deploy.sh
python3 -m http.server -d dist 8000
# Test at localhost:8000 before deploying
```

### 4. Monitor Repository Size
```bash
# Check repo size
git count-objects -vH

# Keep images and large files in Git LFS or external storage
```

### 5. Use .nojekyll File
GitHub Pages uses Jekyll by default. Disable it:
```bash
# Add to your build
touch dist/.nojekyll
```

This is important for files/folders starting with `_`

---

## Example: Complete Workflow

### Initial Setup (One Time)

```bash
# 1. Create repository on GitHub
# 2. Clone it locally
git clone https://github.com/yourusername/tei-viewer.git
cd tei-viewer

# 3. Add your code
# (your existing TEI viewer code)

# 4. Create GitHub Actions workflow
mkdir -p .github/workflows
# Copy the deploy.yml from Option 2 above

# 5. Commit and push
git add .
git commit -m "Initial commit"
git push origin main

# 6. Enable GitHub Pages
# Go to Settings → Pages → Select "GitHub Actions"
```

### Regular Updates

```bash
# Make changes to your code
# ...

# Commit and push
git add .
git commit -m "Update TEI viewer"
git push origin main

# GitHub Actions automatically rebuilds and deploys!
# Check the Actions tab to watch progress
```

---

## Advantages of GitHub Pages for Academic Projects

1. **Free and Permanent** - Perfect for academic/research projects
2. **Version Control** - All changes tracked in Git
3. **Collaboration** - Easy for teams to contribute
4. **Transparency** - Open source by default
5. **Citeable** - Stable URLs for publications
6. **No Maintenance** - GitHub handles servers and updates

---

## Recommended Setup for TEI Viewer

**For this project, I recommend:**

1. **Use GitHub Actions** (automated deployment)
2. **Keep source on `main` branch**
3. **Deploy to `gh-pages` via Actions**
4. **Optional:** Add custom domain for professional appearance

This gives you:
- ✅ Automatic deployment
- ✅ Clean separation (source vs. built files)
- ✅ Easy rollback (just revert commits)
- ✅ Professional workflow

---

## Quick Reference Commands

```bash
# Manual deployment (quick and dirty)
./deploy.sh
git checkout --orphan gh-pages
git rm -rf . && cp -r dist/* . && git add . && git commit -m "Deploy"
git push origin gh-pages --force
git checkout main

# Check deployment status
git log origin/gh-pages  # See deployment history

# View your site
open https://yourusername.github.io/tei-viewer/

# Update custom domain
echo "www.yourdomain.com" > dist/CNAME

# Force rebuild (if using Actions)
git commit --allow-empty -m "Trigger rebuild"
git push origin main
```

---

## Next Steps

1. Choose your deployment method (manual or Actions)
2. Build your site: `./deploy.sh`
3. Deploy using one of the methods above
4. Enable GitHub Pages in Settings
5. Wait 1-2 minutes
6. Visit your live site!

**Your site URL will be:**
```
https://yourusername.github.io/tei-viewer/
```

(Replace `yourusername` with your GitHub username)

---

## Support Resources

- **GitHub Pages Docs:** https://docs.github.com/en/pages
- **GitHub Actions Docs:** https://docs.github.com/en/actions
- **Troubleshooting Guide:** https://docs.github.com/en/pages/getting-started-with-github-pages/troubleshooting-404-errors-for-github-pages-sites

Happy deploying! 🚀