#!/bin/bash

# Update script for TEI Viewer
# Scans projects directory and rebuilds the application

echo "🔍 Scanning projects directory for XML files..."
./build_page_list.sh

echo ""
echo "📁 Copying projects to public directory..."
cp -r projects public/

echo ""
echo "🔨 Building application..."
trunk build

echo ""
echo "✅ Update complete!"
echo "Run 'trunk serve' to start the development server"
