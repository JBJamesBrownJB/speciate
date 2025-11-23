#!/bin/bash
# NAPI Binary Freshness Check
# Ensures .node binary is up-to-date with Rust source files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SIMULATION_DIR="$(dirname "$SCRIPT_DIR")"

# Find the NAPI binary
NODE_BINARY=$(find "$SIMULATION_DIR" -maxdepth 1 -name "*.node" | head -n 1)

if [ -z "$NODE_BINARY" ]; then
    echo "❌ ERROR: No .node binary found in $SIMULATION_DIR"
    echo "   Run: npm run build"
    exit 1
fi

echo "🔍 Checking NAPI binary freshness..."
echo "   Binary: $NODE_BINARY"

# Get binary modification time
BINARY_TIME=$(stat -c %Y "$NODE_BINARY" 2>/dev/null || stat -f %m "$NODE_BINARY" 2>/dev/null)

# Check if any Rust source file is newer than the binary
STALE=0
while IFS= read -r -d '' rust_file; do
    FILE_TIME=$(stat -c %Y "$rust_file" 2>/dev/null || stat -f %m "$rust_file" 2>/dev/null)

    if [ "$FILE_TIME" -gt "$BINARY_TIME" ]; then
        echo "⚠️  Stale binary detected!"
        echo "   Newer file: $rust_file"
        STALE=1
        break
    fi
done < <(find "$SIMULATION_DIR/src" -name "*.rs" -print0)

if [ $STALE -eq 1 ]; then
    echo ""
    echo "❌ ERROR: NAPI binary is out of date"
    echo "   Rust source files have been modified since last build"
    echo ""
    echo "   To fix, run:"
    echo "   cd $SIMULATION_DIR && npm run build"
    echo ""
    exit 1
fi

echo "✅ NAPI binary is up-to-date"
exit 0
