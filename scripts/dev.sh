#!/bin/bash
# Development mode - Build and run Electron desktop app + Dev Tools UI
# Opens both the main game window and the dev tools UI in parallel

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "📦 Installing dependencies..."
echo "=========================================="
cd "$ROOT_DIR/apps/dev-ui"
if [ ! -d "node_modules" ] || [ ! -f "node_modules/.bin/vite" ]; then
  echo "  Dev-UI dependencies missing, installing..."
  npm install
else
  echo "  Dev-UI dependencies OK"
fi

cd "$ROOT_DIR/apps/portal"
if [ ! -d "node_modules" ]; then
  echo "  Portal dependencies missing, installing..."
  npm install
else
  echo "  Portal dependencies OK"
fi

cd "$ROOT_DIR/apps/simulation"
if [ ! -d "node_modules" ]; then
  echo "  Simulation npm dependencies missing, installing..."
  npm install
else
  echo "  Simulation dependencies OK"
fi

echo ""
echo "=========================================="
echo "🔧 Building NAPI module (debug mode)..."
echo "=========================================="
npm run build:debug

echo ""
echo "=========================================="
echo "🚀 Starting development environment..."
echo "=========================================="
echo ""
echo "  Main App:      http://localhost:5173"
echo "  Dev Tools UI:  http://localhost:5174"
echo ""

# Run both dev-ui and portal in parallel
# Use trap to kill all background processes on exit
trap 'kill $(jobs -p) 2>/dev/null' EXIT

# Start dev-ui server in background
cd "$ROOT_DIR/apps/dev-ui"
npm run dev &

# Give dev-ui time to start
sleep 2

# Start portal/Electron (this blocks until user quits)
cd "$ROOT_DIR/apps/portal"
npm run dev
