#!/bin/bash
# Development mode - Build and run Electron desktop app + Dev Tools UI
# Opens both the main game window and the dev tools UI in parallel

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "🔧 Building simulation (debug mode)..."
echo "=========================================="
cd "$ROOT_DIR/apps/simulation"
cargo build --features dev-tools

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

# Start portal/Electron (this blocks until user quits)
cd "$ROOT_DIR/apps/portal"
npm run dev
