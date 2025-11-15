# Development Scripts

Three simple scripts for the complete development workflow.

## Quick Start

```bash
# Development - build and run Electron app
./scripts/dev.sh

# Testing - run all tests
./scripts/test.sh

# Release - build and package for distribution
./scripts/release.sh
```

---

## Scripts

### `dev.sh` - Development Mode

Builds and runs the full development environment:

```bash
./scripts/dev.sh
```

**What it does:**
1. Builds Rust simulation with dev-tools (debug mode)
2. Launches **Dev Tools UI** at http://localhost:5174 (React app)
3. Launches **Electron app** (main game window)

**Opens two windows:**
- **Main Game** - Electron window with PixiJS renderer
- **Dev Tools UI** - Web browser with spawn controls, trial selector, state display

**Use when:** You want to run the full desktop application with developer tools.

**Notes:**
- Both servers run in parallel (dev-ui + portal)
- Simulation runs as Electron subprocess (no separate terminal needed)
- Hot reload enabled for both UIs
- Ctrl+C kills both processes cleanly

---

### `test.sh` - Run All Tests

Runs the complete test suite across all project components:

```bash
./scripts/test.sh
```

**What it does:**
1. Runs Rust simulation tests (`cargo test --features dev-tools`)
2. Runs Portal TypeScript tests (`npm test`)
3. Runs dev-ui integration tests (IPC + trial loading)

**Use when:**
- Before committing changes
- Verifying nothing broke
- CI/CD pipeline

**Expected output:** ~201 Rust tests + TypeScript tests + 2 integration tests

---

### `release.sh` - Build for Distribution

Builds and packages the application for distribution:

```bash
./scripts/release.sh
```

**What it does:**
1. Builds Rust simulation (release mode, optimized)
2. Builds TypeScript portal frontend (production build)
3. Packages with electron-builder (creates installers)

**Creates:**
- Windows: `.exe` installer
- macOS: `.dmg` installer
- Linux: `.AppImage`

**Output location:** `apps/portal/dist-electron/`

**Use when:**
- Creating builds for distribution
- Testing release performance
- Preparing for Steam upload

---

## Development Workflow

### First Time Setup

```bash
# Install Rust dependencies
cd apps/simulation
cargo build

# Install Portal dependencies
cd ../portal
npm install

# Install dev-ui dependencies
cd ../dev-ui
npm install
```

### Daily Development

```bash
# Start development environment
./scripts/dev.sh

# In another terminal: run tests
./scripts/test.sh

# In apps/dev-ui: run integration tests manually
cd apps/dev-ui
npm run test:ipc      # Test single creature spawning
npm run test:trial    # Test trial loading
```

### Pre-Commit

```bash
# Always run tests before committing
./scripts/test.sh
```

### Release

```bash
# Build and package for distribution
./scripts/release.sh

# Upload dist-electron/*.exe, *.dmg, or *.AppImage
```

---

## Script Details

### Paths

All scripts:
- Are designed to be run from project root
- Use `set -e` to exit on first error
- Resolve paths relative to script location (work from anywhere)

### Build Modes

| Script | Simulation | Portal | Purpose |
|--------|-----------|--------|---------|
| `dev.sh` | Debug + dev-tools | Development | Fast builds, debugging |
| `test.sh` | Debug + dev-tools | Testing | Run full test suite |
| `release.sh` | Release (optimized) | Production | Distribution builds |

### Performance

- **dev.sh:** ~2-5 seconds (incremental builds)
- **test.sh:** ~10-30 seconds (depends on test count)
- **release.sh:** ~30-120 seconds (full optimization)

---

## Troubleshooting

**Port 5173 already in use:**
```bash
# Kill existing dev server
pkill -f vite
# Or find and kill the process
lsof -ti:5173 | xargs kill -9
```

**Simulation binary not found:**
```bash
# Rebuild simulation
cd apps/simulation
cargo build --features dev-tools
```

**npm dependencies out of date:**
```bash
cd apps/portal && npm install
cd ../dev-ui && npm install
```

---

## See Also

- `/workspace/CLAUDE.md` - Project-wide development principles
- `/workspace/apps/dev-ui/README.md` - Integration test documentation
- `/workspace/docs/architecture/electron-architecture.md` - IPC architecture
