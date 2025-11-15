#!/bin/bash
# Run all tests: Rust simulation + TypeScript portal + dev-ui integration

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "🦀 Running Rust simulation tests..."
echo "=========================================="
cd "$ROOT_DIR/apps/simulation"
cargo test --features dev-tools

echo ""
echo "=========================================="
echo "📘 Running Portal TypeScript tests..."
echo "=========================================="
cd "$ROOT_DIR/apps/portal"
npm test

echo ""
echo "=========================================="
echo "🧪 Running dev-ui integration tests..."
echo "=========================================="
cd "$ROOT_DIR/apps/dev-ui"
npm test

echo ""
echo "=========================================="
echo "✅ All tests passed!"
echo "=========================================="
