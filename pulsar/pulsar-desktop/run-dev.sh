#!/bin/bash
# Quick start script for Pulsar development

set -e

echo "🚀 Starting Pulsar Desktop in development mode..."
echo ""

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing npm dependencies..."
    npm install
    echo ""
fi

# Start Tauri dev server
echo "🔧 Starting Vite + Tauri..."
echo "   - Frontend: http://localhost:5173"
echo "   - Tauri desktop app will launch automatically"
echo ""

npm run tauri dev
