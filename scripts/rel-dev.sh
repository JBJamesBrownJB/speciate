#!/bin/bash
# Development mode with RELEASE simulation build
# Use this for performance testing with optimized Rust binary

set -e

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
echo "🔧 Building NAPI module (RELEASE mode)..."
echo "=========================================="
npm run build

echo ""
echo "=========================================="
echo "🚀 Starting development environment (RELEASE)..."
echo "=========================================="
echo ""
echo "  Main App:      http://localhost:5173"
echo "  Dev Tools UI:  http://localhost:5174"
echo "  Simulation:    RELEASE BUILD (optimized)"
echo ""

trap 'kill $(jobs -p) 2>/dev/null' EXIT

cd "$ROOT_DIR/apps/dev-ui"
npm run dev &

# Give dev-ui time to start
sleep 2

cd "$ROOT_DIR/apps/portal"
RUST_BUILD_TYPE=release npm run dev
