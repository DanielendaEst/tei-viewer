#!/bin/bash

# TEI-XML Papyrus Viewer - Startup Script
# This script sets up and runs the development server

set -e

echo "🚀 TEI-XML Papyrus Viewer - Starting..."
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
    echo ""
fi

# Check if dist directory exists and has the right structure
if [ ! -d "dist/public/projects" ]; then
    echo "📦 First time build detected..."
    echo "   Building application..."

    # Do an initial build
    trunk build

    echo "✅ Build complete!"
    echo ""
fi

# Start the server
echo "🌐 Starting development server..."
echo ""
echo "   Server will be available at: http://127.0.0.1:8080"
echo ""
echo "   Features available:"
echo "   • Diplomatic & Translation editions"
echo "   • Image-text synchronization"
echo "   • Interactive highlighting"
echo "   • Semantic markup visualization"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""
echo "─────────────────────────────────────────────"
echo ""

trunk serve
