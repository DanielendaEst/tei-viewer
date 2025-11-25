#!/bin/bash

# TEI-XML Papyrus Viewer - Production Deployment Script
# This script builds the application for production deployment

set -e

echo "🔨 TEI-XML Papyrus Viewer - Production Build"
echo ""

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ Error: trunk is not installed"
    echo ""
    echo "Please install trunk with:"
    echo "  cargo install trunk"
    echo ""
    exit 1
fi

# Remove Trunk.toml if it exists (causes issues)
if [ -f "Trunk.toml" ]; then
    echo "⚠️  Removing Trunk.toml (causes build issues)..."
    rm -f Trunk.toml
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
        echo "📦 Setting up public directory structure..."
        mkdir -p public

        if [ -d "projects" ]; then
            echo "📁 Copying projects to public directory..."
            cp -r projects public/
            echo "✅ Projects copied successfully!"
        else
            echo "⚠️  Warning: projects/ directory not found"
            echo "   Please ensure your project data is in the projects/ folder"
        fi
    fi
fi
echo ""

# Clean previous build
if [ -d "dist" ]; then
    echo "🧹 Cleaning previous build..."
    rm -rf dist
fi

# Build for production
echo "🏗️  Building production bundle..."
echo "   Note: Using dev build due to wasm-opt compatibility issues"
echo ""
trunk build

echo ""
echo "✅ Production build complete!"
echo ""
echo "📦 Deployment files are in: ./dist/"
echo ""
echo "Next steps:"
echo "  1. Upload the ./dist/ directory to your web server"
echo "  2. Configure your server to serve static files"
echo "  3. Ensure proper MIME types for .wasm files"
echo ""
echo "Example deployment commands:"
echo "  • Simple HTTP server:  python3 -m http.server -d dist 8080"
echo "  • Nginx:              Copy dist/* to /var/www/html/"
echo "  • Apache:             Copy dist/* to /var/www/html/"
echo "  • Netlify/Vercel:     Deploy the dist/ directory"
echo ""
