# Speciate - DNA-Driven A-Life Simulation

**A single-player desktop game featuring DNA-driven artificial life, emergent ecosystems, and systemic survival gameplay.**

**Platform:** Windows, Mac, Linux (Electron desktop application)
**Target:** Steam Early Access Q2 2026
**Status:** Phase 1 Development (Sandbox Mode)

> See [docs/project-spec.md](docs/project-spec.md) for complete technical specification
> See [docs/strategy/biz-strategy.md](docs/strategy/biz-strategy.md) for business model and phase gates

---

## What is Speciate?

**Speciate** is an artificial life simulation where **DNA drives everything**. Hundreds of autonomous creatures evolve, compete, and adapt in a procedurally generated alien ecosystem. No scripted behaviors, no hardcoded NPCs—complex patterns emerge from simple genetic primitives.

**Core Gameplay (Phase 1 - Sandbox):**
- 🧬 **DNA-Driven Evolution** - All creature behavior flows from genetic code
- 🌍 **Explore** - Navigate vast alien world with fog of war
- 🐾 **Tame & Breed** - Domesticate creatures, selective breeding for traits
- 🔬 **Experiment** - Manipulate ecosystem, observe emergent dynamics
- 💀 **Survive** - Gather biomass, craft tools, avoid predators

**Future Content (Phase 1.5 - Post-Launch):**
- 📖 **Story Campaign** - Find and rescue daughter across dangerous planet
- 🦖 **Advanced Taming** - Harpoon capture, DNA cloning, creature commands
- 🐒 **Drongo Species** - Intelligent tool-users with social learning
- ⚔️ **Endgame Challenge** - Navigate Karg territory using tamed creatures

---

## Development Phase

### Current: Phase 1 (6-9 months)

**Goal:** Prove DNA-driven A-Life concept is fun, build community, generate revenue

**Deliverables:**
- Standalone desktop game (Steam Early Access)
- DNA system (size, speed, perception, aggression, social learning)
- Sandbox mode (observe, breed, manipulate ecosystem)
- Procedural world generation
- Save/load system with Steam Cloud support

### Next: Phase 1.5 (Post-Launch)

**Goal:** Add emotional depth and retention through narrative campaign

**Deliverables:**
- Daughter rescue story mode
- Taming system (beacon zones, harpoon, cloning)
- Drongo species + social learning mechanics
- Creature commands (Thumper, herding)
- Karg territory gauntlet

### Future: Phase 2 (2027+)

**Goal:** Expand to web MMO if Phase 1 succeeds

**Deliverables:**
- Browser-based multiplayer
- Player economy (DNA ownership, biomass trading)
- Speciation events
- Conservation mechanics

**Status:** Archived pending Early Access success (see [docs/strategy/biz-strategy.md](docs/strategy/biz-strategy.md))

---

## Architecture

**Current (Phase 1): Electron Hybrid Desktop**

```
┌─────────────────────────────────────────────────────────────┐
│                  ELECTRON APPLICATION                        │
├──────────────────────────┬───────────────────────────────────┤
│  RUST SUBPROCESS         │  FRONTEND (PixiJS)               │
│  (Bevy ECS)              │                                   │
│                          │                                   │
│  Simulation Loop:        │  app.ticker (60 FPS):            │
│  • AI & Decision Making  │  • Receive state-update events   │
│  • Steering Behaviors    │  • Update sprite positions       │
│  • Physics Integration   │  • Render frame                  │
│                          │                                   │
│  stdout MessagePack:     │                                   │
│  • Stream at tick rate   │  Main Process → Renderer         │
│  • Write to stdout ──────┼──> Decode and forward            │
└──────────────────────────┴───────────────────────────────────┘
```

**Benefits:**
- $228k/year server costs eliminated
- No network complexity (interpolation, quantization, sync)
- Full f32 precision coordinates
- Simple stdio IPC (no shared memory complexity)

**See:** [docs/architecture/electron-architecture.md](docs/architecture/electron-architecture.md)

---

## Getting Started

### Prerequisites

**Local Development Requirements:**
- **Rust** 1.75+ (`rustc --version`) - For simulation backend
- **Node.js** 18+ (`node --version`) - For Electron + frontend
- **npm** 10+ (`npm --version`)
- **System dependencies:**
  - **Linux:**
    - `perf` (performance metrics) - Install: `sudo apt-get install linux-tools-generic linux-tools-$(uname -r)`
    - **Hardware counter permissions** (required for dev-tools):
      ```bash
      sudo sysctl -w kernel.perf_event_paranoid=1
      ```
      - **Why:** Allows CPU performance counters (cycles, cache misses, IPC)
      - **When:** Run this if you see `Permission denied (os error 13)` errors
      - **Verify:** `cat /proc/sys/kernel/perf_event_paranoid` (should show 1 or lower)
      - **Permanent:** `echo "kernel.perf_event_paranoid=1" | sudo tee -a /etc/sysctl.conf`
    - Electron bundles Chromium
  - **macOS:** None (Electron bundles Chromium)
  - **Windows:** None (Electron bundles Chromium)

### Quick Start

**First-Time Setup:**

```bash
cd apps/portal
npm run setup  # Installs deps + builds debug Rust + frontend (2-3 min)
npm run dev    # Launches app with hot reload
```

---

## Development Workflows

### 🎨 Frontend Changes (PixiJS/TypeScript) - Instant Feedback

```bash
npm run dev  # Start once, leave running
```

Edit any `.ts` or `.tsx` file → **Changes appear in <1 second** (Vite HMR)

**Example:** Change sprite colors, UI layouts, camera controls → Instant visual update!

---

### 🦀 Rust Changes (Simulation) - Fast Iteration

```bash
# 1. Edit Rust code (simulation behavior, physics, etc.)
# 2. Rebuild debug binary (30 seconds)
npm run dev:rust

# 3. Restart Electron (Ctrl+R or relaunch npm run dev)
```

**Speed:** 30 sec rebuild → See simulation changes visually in frontend

**Example:** Modify creature speed, steering behavior, spawning logic → Quick feedback loop!

---

### 📦 Production Build - Final Testing

```bash
npm run build          # Optimized Rust + frontend (3-5 min)
npm run package:linux  # Create standalone .AppImage/.deb
```

**When to use:**
- Pre-commit validation
- Performance testing (release builds are ~2x faster)
- Creating installers for distribution

---

## Command Reference

```bash
# First-time setup
npm run setup              # Install deps, build debug Rust, build frontend

# Development
npm run dev                # Start Vite dev server + Electron (hot reload)
npm run dev:vite           # Vite dev server only (for browser testing)
npm run dev:electron       # Electron only (if Vite already running)
npm run dev:rust           # Rebuild debug Rust binary (30 sec)

# Production builds
npm run build              # Build optimized Rust + frontend
npm run build:rust         # Rust release build only (3-5 min)
npm run build:frontend     # Frontend build only

# Packaging
npm run package            # Build + package for current platform
npm run package:linux      # Linux .deb + .AppImage
npm run package:mac        # macOS .dmg
npm run package:win        # Windows .exe installer

# Testing
npm test                   # Frontend tests
npm run type-check         # TypeScript validation
cd ../simulation && cargo test  # Rust tests
```

---

## Troubleshooting

**White screen on launch:**
```bash
# Build debug Rust binary (development)
npm run dev:rust

# Or build release binary (production)
npm run build:rust
```

**"Cannot connect to Vite":**
```bash
# Make sure both processes are running
npm run dev  # Starts Vite + Electron in parallel
```

**Slow Rust compilation:**
- Use `npm run dev:rust` (debug builds, 30 sec)
- Only use `npm run build:rust` for production (3-5 min)

**"No creatures rendering"**
- Check browser console for JavaScript errors
- Verify dist/ folder exists: `ls apps/portal/dist`
- Rebuild frontend: `npm run build`

**"Cargo build fails"**
- Check Rust version: `rustc --version` (need 1.75+)
- Clean and rebuild: `cargo clean && cargo build --release`

**"npm install fails"**
- Check Node.js version: `node --version` (need 18+)
- Clear npm cache: `npm cache clean --force && npm install`

**"Permission denied (os error 13)" / Hardware counter errors:**
```
Failed to build CPU_CYCLES counter: Permission denied (os error 13)
⚠️  Failed to initialize hardware counters: Permission denied (os error 13)
   Falling back to disabled state
```
- **Fix:** Set kernel permission: `sudo sysctl -w kernel.perf_event_paranoid=1`
- **Verify:** `cat /proc/sys/kernel/perf_event_paranoid` (should show 1 or lower)
- **Note:** Simulation continues running without hardware metrics (graceful fallback)

---

## Project Structure

```
/workspace
├── apps/
│   ├── simulation/         # Rust/Bevy ECS simulation engine
│   └── portal/             # PixiJS frontend + Electron wrapper
│       ├── electron/       # Electron main process + preload
│       ├── src/            # TypeScript frontend (PixiJS)
│       └── dist/           # Vite build output
├── docs/
│   ├── strategy/           # Business model, game goal
│   ├── architecture/       # Electron patterns, performance
│   ├── biology/            # DNA design, species, zoologist notes
│   ├── gameplay/           # Taming, combat, progression
│   └── project-spec.md     # Complete technical specification
└── .claude/
    ├── agents/             # AI development team definitions
    ├── commands/           # Custom slash commands
    └── hooks/              # Pre-commit validation scripts
```

---

## Key Design Principles

### 1. DNA-Driven Design (MANDATORY)

**The DNA is the creature. Everything else is just expression.**

- **All traits flow from genes:** Size, speed, perception, aggression, social learning
- **Complex behaviors emerge:** Sociality emerges from low aggression + high flocking
- **Systemic trade-offs:** Large + fast = massive energy consumption → starvation
- **No hardcoded values:** Every creature unique via genetic variation

**See:** [docs/biology/ideas/dna-driven-design.md](docs/biology/ideas/dna-driven-design.md)

### 2. Test-Driven Development (MANDATORY)

**Tests exist to catch breaking changes. They're worthless if you don't use them.**

- **Before ANY code change:** Run `npm test` to verify current state
- **Write tests FIRST** for new functionality
- **Run tests IMMEDIATELY** after changes
- **Never skip tests** because "it's a small change"

**Current:** 196 tests passing (Portal + Simulation)

### 3. Emergence Over Scripting

**Systems, not scripts:**
- ❌ "Sociality" gene → ✅ Emerges from: personal_space + flocking + low aggression
- ❌ Scripted boss fights → ✅ Systemic gauntlet challenges
- ❌ Hardcoded helper NPCs → ✅ DNA-driven social species (Drongos)

---

## AI Development Team

Speciate uses specialized AI agents (via Claude Code) for development:

### Core Engineering
- **architect-andy** - Technical architecture, system design, performance analysis
- **rusty-ron** - Rust simulation, A-Life systems, ECS implementation
- **ecs-eddy** - ECS optimization, performance profiling, Data-Oriented Design
- **instrumentation-ian** - Linux performance analysis, telemetry pipelines, empirical validation
- **frontend-fanny** - PixiJS rendering, UI/UX, client optimization

### Domain Experts
- **zoologist-tom** - Ecosystem design, biology validation, DNA traits
- **botanist-betsy** - Plant biology, growth systems
- **environment-eddy** - Procedural generation, biomes, terrain
- **gamification-garry** - Game design, balance, player motivation
- **narrative-nancy** - Story design, quests, campaign structure (Phase 1.5+)

### Distribution & QA
- **steam-steve** - Steam integration, achievements, cloud saves, workshop
- **playtest-petra** - E2E testing, gameplay validation, UX evaluation
- **qa-karen** - Pre-merge reviews, security, standards

### Project Management
- **pm-pam** - Sprint management, task coordination, agile workflow

---

## Recent Development

**Recently Completed:**
- ✅ ECS optimization with Rayon parallelization (6.3x movement speedup)
- ✅ Vision system refactor (split queries, 2x capacity)
- ✅ Energy-modulated personal space (biological hunger mechanics)
- ✅ Target radius seeking (edge-to-edge arrival)
- ✅ Perlin noise locomotion (organic movement jitter)
- ✅ Code quality improvements and technical debt cleanup

**See:** `sprint_summaries/` folder for detailed development history

---

## Resources

### Project Documentation
- [docs/project-spec.md](docs/project-spec.md) - Complete technical specification
- [docs/strategy/biz-strategy.md](docs/strategy/biz-strategy.md) - Business model & phase gates
- [docs/gameplay/ideas/end game/goal.md](docs/gameplay/ideas/end game/goal.md) - Game narrative & design
- [docs/architecture/electron-architecture.md](docs/architecture/electron-architecture.md) - Current architecture
- [docs/biology/ideas/dna-driven-design.md](docs/biology/ideas/dna-driven-design.md) - Core design principle
- [CLAUDE.md](CLAUDE.md) - TDD requirements & DNA enforcement

### Technology Documentation
- [Electron](https://www.electronjs.org/) - Desktop app framework
- [Bevy ECS](https://bevyengine.org/) - Entity Component System
- [Pixi.js 8.x](https://pixijs.com/8.x/guides) - 2D WebGL renderer
- [Rust Book](https://doc.rust-lang.org/book/) - Learning Rust
- [TypeScript Handbook](https://www.typescriptlang.org/docs/) - TypeScript guide

---

## Contributing

**Current Focus:** Phase 1 (Steam Early Access sandbox)

**Priorities:**
1. DNA system implementation
2. Player interaction UI
3. World generation
4. Steam integration & polish

**Deferred to Phase 1.5:**
- Narrative campaign (daughter rescue)
- Taming system (beacon, harpoon, cloning)
- Drongo species
- Creature commands

**Deferred to Phase 2:**
- Multiplayer/MMO features
- Player economy
- Speciation events

---

## License

[TODO: Add license]

---

**The DNA is the creature. Everything else is just expression.**
