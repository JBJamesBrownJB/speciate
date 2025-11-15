#!/bin/bash
# Build and package for distribution
# Creates platform-specific installers (.exe, .dmg, .AppImage)

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "🦀 Building simulation (release mode)..."
echo "=========================================="
cd "$ROOT_DIR/apps/simulation"
cargo build --release

echo ""
echo "=========================================="
echo "📦 Building portal frontend..."
echo "=========================================="
cd "$ROOT_DIR/apps/portal"
npm run build

echo ""
echo "=========================================="
echo "📦 Packaging with electron-builder..."
echo "=========================================="
npm run package

echo ""
echo "=========================================="
echo "✅ Release build complete!"
echo "=========================================="
echo ""
echo "Distribution packages created in:"
echo "  apps/portal/dist-electron/"
echo ""
echo "Binaries:"
echo "  Simulation: apps/simulation/target/release/speciate"
echo "  Portal:     apps/portal/dist/"
