# Dev UI - Developer Tools Interface

Web-based developer tools UI for the Speciate simulation. Provides a visual interface to spawn creatures, load trial templates, and monitor simulation state in real-time.

## Features

- **Spawn Creatures** - Spawn individual creatures at specific coordinates
- **Trial Selector** - Load pre-configured trial scenarios from TOML files
- **State Display** - View real-time simulation state (tick, creature count)
- **IPC Integration** - Communicates with simulation via Electron IPC

## Quick Start

From project root:

```bash
./scripts/dev.sh
```

This launches:
- **Main App** (Electron game window) at http://localhost:5173
- **Dev Tools UI** (this app) at http://localhost:5174

## Manual Usage

```bash
# Install dependencies (first time)
cd apps/dev-ui
npm install

# Run dev server
npm run dev
```

Then open http://localhost:5174 in your browser.

## Components

### DevToolsApp.tsx
Main application container. Manages layout and coordinates between sub-components.

### SpawnForm.tsx
Form to spawn creatures at specific X/Y coordinates. Sends `DevSpawnCreature` command to simulation.

### TrialSelector.tsx
Dropdown to select and load trial templates (e.g., "default-spawn-baseline", "crowd-navigation"). Sends `DevLoadTrial` command.

### StateDisplay.tsx
Shows real-time simulation state:
- Current tick number
- Creature count
- Tick rate (Hz)

## Integration Tests

The dev-ui folder also contains integration tests:

```bash
npm run test:ipc      # Test single creature spawn
npm run test:trial    # Test trial loading
npm test              # Run both tests
```

These tests spawn the simulation binary directly (no Electron) to verify IPC communication works.

## Architecture

```
┌─────────────────────────────────────────┐
│  Dev UI (React + Vite)                  │
│  http://localhost:5174                  │
│                                         │
│  ┌─────────────┐  ┌─────────────────┐  │
│  │ SpawnForm   │  │ TrialSelector   │  │
│  └─────┬───────┘  └────────┬────────┘  │
│        │                   │            │
│        └───────┬───────────┘            │
│                ▼                        │
│         window.electron.send()          │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  Electron Main Process                  │
│  (apps/portal/electron/main.cjs)        │
│                                         │
│  simulation.stdin.write(command)        │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  Rust Simulation (stdio subprocess)     │
│  Reads commands from stdin              │
│  Executes: spawn creatures, load trials │
└─────────────────────────────────────────┘
```

## Building for Production

```bash
npm run build
```

Output: `dist/` directory with static HTML/JS/CSS.

## Requirements

- Node.js 18+
- React 18
- Vite 7
- TypeScript 5

## See Also

- [`AGENTS.md`](./AGENTS.md) - dev-ui area guide (traps, codegen coupling, timings contract)
- [`../../scripts/README.md`](../../scripts/README.md) - development workflow
- [`../../docs/architecture/electron-architecture.md`](../../docs/architecture/electron-architecture.md) - Electron / NAPI IPC architecture
- Trial template TOMLs are generated from [`../../apps/simulation/specs/`](../../apps/simulation/specs/)
