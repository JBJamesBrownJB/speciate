# Speciate - DNA-Driven A-Life Simulation

**A single-player desktop game featuring DNA-driven artificial life, emergent ecosystems, and systemic survival gameplay.**

**Platform:** Windows, Mac, Linux (Tauri desktop application)
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

**Current (Phase 1): Tauri Hybrid Desktop**

```
┌─────────────────────────────────────────────────────────────┐
│                    TAURI APPLICATION                         │
├──────────────────────────┬───────────────────────────────────┤
│  RUST BACKEND            │  FRONTEND (PixiJS)               │
│  (Bevy ECS)              │                                   │
│                          │                                   │
│  FixedUpdate (20 Hz):    │  app.ticker (90 FPS):            │
│  • AI & Decision Making  │  • invoke('get_game_state')      │
│  • Steering Behaviors    │  • Update sprite positions       │
│  • Pathfinding           │  • Render frame                  │
│                          │                                   │
│  Update (90 Hz):         │                                   │
│  • Physics Integration   │                                   │
│  • Snapshot to Queue ────┼──> Lock-Free IPC                 │
└──────────────────────────┴───────────────────────────────────┘
```

**Benefits:**
- $228k/year server costs eliminated
- No network complexity (interpolation, quantization, sync)
- Full f32 precision coordinates
- Faster development and iteration

**See:** [docs/architecture/tauri-architecture.md](docs/architecture/tauri-architecture.md)

---

## Getting Started

### Prerequisites

- **Rust** 1.70+ (with Cargo)
- **Node.js** 22.12+ (for Vite 7 ESM support)
- **npm** 10+
- **Tauri CLI** (install via: `cargo install tauri-cli`)
- **VS Code** with Dev Containers extension (recommended)

### Quick Start (Current Development)

**Note:** Tauri migration is Sprint 7. Current setup uses NATS streaming (will be removed).

**For now, start the multi-service stack:**

Open **5 terminal windows**:

```bash
# Terminal 1: NATS (temporary, will be removed in Sprint 7)
cd infrastructure/local && docker compose up

# Terminal 2: Broadcaster (temporary, will be removed in Sprint 7)
cd apps/broadcaster && npm run dev

# Terminal 3: Simulation (with dev commands for admin UI)
cd apps/simulation && cargo run --features dev-commands

# Terminal 4: Portal (frontend - will migrate to Tauri)
cd apps/portal && npm run dev

# Terminal 5: Admin Dev UI
cd apps/admin-dev-ui && python3 -m http.server 8000
```

**Service URLs:**
- **Portal:** http://localhost:3000
- **Admin UI:** http://localhost:8000
- **Broadcaster:** ws://localhost:8080
- **NATS Monitor:** http://localhost:8222

**Quick Test:**
1. Wait for all services to start (~30 seconds)
2. Open Admin UI: http://localhost:8000
3. Click "Two Seekers Intercept" scenario
4. Open Portal: http://localhost:3000
5. Watch creatures spawn and interact!

---

### Post-Sprint 7 (Tauri Unified App)

**Coming Soon:**

```bash
# Single command to run everything
cd apps/desktop
npm install
npm run tauri dev
```

One window, one process, no NATS, no complexity.

---

## Project Structure

```
/workspace
├── apps/
│   ├── simulation/         # Rust/Bevy ECS simulation engine
│   ├── portal/             # PixiJS frontend (migrating to Tauri)
│   ├── broadcaster/        # Node.js WebSocket (archiving in Sprint 7)
│   ├── admin-dev-ui/       # Dev testing UI
│   └── ledger/             # Economy service (Phase 2)
├── docs/
│   ├── strategy/           # Business model, game goal
│   ├── architecture/       # Tauri, streaming (archived), patterns
│   ├── biology/            # DNA design, species, zoologist notes
│   ├── gameplay/           # Taming, combat, progression
│   └── project-spec.md     # Complete technical specification
├── infrastructure/
│   └── local/              # Docker Compose for NATS (temporary)
└── .claude/
    ├── agents/             # AI development team definitions
    └── spec/               # Architecture standards
```

---

## Key Design Principles

### 1. DNA-Driven Design (MANDATORY)

**The DNA is the creature. Everything else is just expression.**

- **All traits flow from genes:** Size, speed, perception, aggression, social learning
- **Complex behaviors emerge:** Sociality emerges from low aggression + high flocking
- **Systemic trade-offs:** Large + fast = massive energy consumption → starvation
- **No hardcoded values:** Every creature unique via genetic variation

**See:** [docs/biology/dna-driven-design.md](docs/biology/dna-driven-design.md)

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
- **backend-simulation-sam** - Rust simulation, A-Life systems, ECS implementation
- **frontend-fanny** - PixiJS rendering, UI/UX, client optimization
- **backend-ledger-larry** - Economy ledger (Phase 2)
- **broadcaster-brian** - WebSocket streaming (archiving Sprint 7)
- **devops-daria** - CI/CD, infrastructure, Terraform

### Domain Experts
- **zoologist-tom** - Ecosystem design, biology validation, DNA traits
- **botanist-betsy** - Plant biology, growth systems
- **environment-eddy** - Procedural generation, biomes, terrain
- **gamification-garry** - Game design, balance, player motivation

### Operations
- **play-test-petra** - E2E testing, gameplay validation
- **qa-karen** - Pre-merge reviews, security, standards
- **pm-pam** - Sprint management, task coordination
- **mr-motivator** - Vision alignment, team focus

---

## Current Sprint Status

**Sprint 6: "Learning to Walk"** ✅ Complete (Nov 6-9, 2025)

**Achievements:**
- Seeking behavior with Reynolds steering
- Territory-based wandering with elastic tether
- Locomotion noise (Perlin-based organic wobble)
- Body radius volumetric physics
- NATS WebSocket support (port 9224)
- Admin portal with live spawning
- Single-gate spawning architecture
- 133 passing tests

**Next: Sprint 7 - Tauri Migration** (5-7 days)

**Goals:**
- Remove NATS, Broadcaster, interpolation code
- Implement Tauri IPC with lock-free snapshot queue
- Dual-tick refactor (20 Hz AI, 90 Hz physics)
- Test 1000 creatures @ 90 FPS
- Cross-platform builds (Windows, Mac, Linux)

---

## Resources

### Project Documentation
- [docs/project-spec.md](docs/project-spec.md) - Complete technical specification
- [docs/strategy/biz-strategy.md](docs/strategy/biz-strategy.md) - Business model & phase gates
- [docs/strategy/goal.md](docs/strategy/goal.md) - Game narrative & design
- [docs/architecture/tauri-architecture.md](docs/architecture/tauri-architecture.md) - Current architecture
- [docs/biology/dna-driven-design.md](docs/biology/dna-driven-design.md) - Core design principle
- [CLAUDE.md](CLAUDE.md) - TDD requirements & DNA enforcement

### Technology Documentation
- [Tauri](https://tauri.app/) - Desktop app framework
- [Bevy ECS](https://bevyengine.org/) - Entity Component System
- [Pixi.js 8.x](https://pixijs.com/8.x/guides) - 2D WebGL renderer
- [Rust Book](https://doc.rust-lang.org/book/) - Learning Rust
- [TypeScript Handbook](https://www.typescriptlang.org/docs/) - TypeScript guide

---

## Contributing

**Current Focus:** Phase 1 (Steam Early Access sandbox)

**Priorities:**
1. DNA system implementation (Sprint 6 Phase 3+)
2. Tauri migration (Sprint 7)
3. Player interaction UI (Sprint 8)
4. World generation (Sprint 9)
5. Steam integration & polish (Sprint 10)

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
