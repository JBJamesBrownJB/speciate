---
description: "Quick launch: Build and run Electron desktop app (simulation + frontend) in dev mode."
allowed-tools:
  - Bash
model: haiku
---

# Run Simulation (Electron Dev Mode)

Launching Electron desktop application in development mode...

## Pre-Flight Checks

```bash
echo "========================================="
echo "PRE-FLIGHT CHECKS"
echo "========================================="
echo ""

# Check if Node.js is available
if ! command -v node &> /dev/null; then
  echo "❌ Node.js not found. Install Node.js 18+ from nodejs.org"
  exit 1
fi

echo "✅ Node.js found ($(node --version))"

# Check if npm is available
if ! command -v npm &> /dev/null; then
  echo "❌ npm not found. Install npm (comes with Node.js)"
  exit 1
fi

echo "✅ npm found ($(npm --version))"

# Check if portal directory exists
if [ ! -d "/home/dev/dev/speciate/apps/portal" ]; then
  echo "❌ apps/portal directory not found. Not in project root?"
  exit 1
fi

echo "✅ apps/portal directory found"

# Check if Rust is available (for simulation build)
if ! command -v cargo &> /dev/null; then
  echo "⚠️  Warning: cargo not found. Rust simulation won't build."
  echo "   Install Rust from: https://rustup.rs/"
else
  echo "✅ Rust found ($(rustc --version))"
fi

echo ""
```

## Build and Launch

```bash
cd /home/dev/dev/speciate/apps/portal

echo "========================================="
echo "LAUNCHING ELECTRON DEV MODE"
echo "========================================="
echo ""
echo "This will:"
echo "1. Build Rust simulation (debug mode) - First time: 2-5 min"
echo "2. Start Vite dev server (frontend)"
echo "3. Launch Electron desktop window"
echo "4. Spawn simulation subprocess (stdio IPC)"
echo ""
echo "⚠️  This is a blocking command. Press Ctrl+C to stop."
echo ""
echo "Starting in 3 seconds..."
sleep 3

# Run Electron dev mode
npm run dev
```

---

## Alternative: Component-by-Component Launch

If you want to launch components separately for debugging:

### Terminal 1: Rust Simulation (Standalone)
```bash
cd /home/dev/dev/speciate/apps/simulation
cargo run --release
```

**Note:** Simulation writes MessagePack frames to stdout. Running standalone will just dump binary to console.

### Terminal 2: Frontend Dev Server (Browser Mode)
```bash
cd /home/dev/dev/speciate/apps/portal
npm run dev:web  # If you have a web-only mode
```

**Note:** Browser mode requires WebSocket IPC (not implemented in Phase 1). Use full `npm run dev` instead.

### Recommended: Full Electron Stack
```bash
cd /home/dev/dev/speciate/apps/portal
npm run dev  # This is the only supported workflow for Phase 1
```

---

## What Happens During Launch

1. **Vite Build:** Frontend TypeScript compiled to JavaScript
2. **Rust Build:** Simulation binary compiled to `apps/simulation/target/debug/speciate`
3. **Electron Main:** Spawns simulation as child process
4. **stdio IPC:** Simulation writes 60 Hz MessagePack frames to stdout
5. **Renderer:** PixiJS receives state updates, renders creatures at 60 FPS

---

## Troubleshooting

**Build fails with "cargo not found":**
- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart terminal, verify: `cargo --version`

**Electron window doesn't open:**
- Check if simulation binary exists: `ls apps/simulation/target/debug/speciate`
- Build manually: `cd apps/simulation && cargo build`
- Check console for errors

**No creatures rendering:**
- Open DevTools (Ctrl+Shift+I / Cmd+Option+I)
- Check Console for JavaScript errors
- Verify dist/ folder exists: `ls apps/portal/dist`
- Rebuild: `npm run build`

**Port conflicts:**
- Default Vite port: 5173
- Kill existing: `lsof -ti:5173 | xargs kill -9`

**Slow first launch:**
- First Rust build: 2-5 minutes (compiles Bevy + dependencies)
- Subsequent builds: 10-20 seconds (incremental compilation)
- Use `--release` for production builds (slower compile, faster runtime)

---

**Architecture:** Electron main process → Rust subprocess → stdout MessagePack (60 Hz) → Renderer

**See:** `docs/architecture/electron-architecture.md` for full design
