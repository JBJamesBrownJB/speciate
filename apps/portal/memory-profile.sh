#!/bin/bash

# Memory Profiling Mode for Electron NAPI
# Runs with --expose-gc flag for manual GC control

echo "Starting Electron with Memory Profiling..."
echo "Memory log will be written to: docs/performance/memory-profile.jsonl"
echo ""
echo "Available commands from dev-ui:"
echo "  - Trigger GC (via IPC)"
echo "  - Take Heap Snapshot (saves .heapsnapshot file)"
echo ""

cd "$(dirname "$0")"

NODE_ENV=development \
ELECTRON_DISABLE_SANDBOX=1 \
electron --expose-gc electron/napi-memory-profile.cjs
