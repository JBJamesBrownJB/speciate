# Speciate

**A million-creature artificial-life engine, where Rust's fearless parallelism meets the web's visual playground.**

[![Target: 1M creatures](https://img.shields.io/badge/target-1M%20creatures-blue)](docs/ROADMAP.md)
[![Linux: 500K achieved](https://img.shields.io/badge/Linux-500K%20achieved-brightgreen)](docs/scale/README.md)
[![Windows: 20K experimental](https://img.shields.io/badge/Windows-20K%20experimental-orange)](docs/scale/README.md)

> These badges are **static placeholders**. Pillar 1 (Prove Scale) builds the cross-OS CI that will make them live, driven from real benchmark runs.

---

## What is Speciate?

Speciate is a **high-performance artificial-life engine and visual sandbox** — a portfolio showcase of Rust × JavaScript systems craft, not a game in production.

It pairs a **Rust + Bevy ECS** simulation backend (deterministic, aggressively parallel, built to push toward a million autonomous creatures) with a **PixiJS / WebGL** frontend (the richest visual ecosystem on earth, with frictionless distribution). The two halves meet at a **zero-copy NAPI `Float32Array` seam** that delivers Rust throughput *and* web reach without the serialization tax that sinks most hybrid designs.

Creatures are DNA-driven: complex behavior emerges from simple genetic primitives, with no scripted NPCs. But the headline here is the **engineering** — the scale, the determinism, and the architecture that makes it possible.

**The showcase argument lives in [docs/architecture/rust-js-thesis.md](docs/architecture/rust-js-thesis.md). The full plan lives in [docs/ROADMAP.md](docs/ROADMAP.md).**

---

## The Four Pillars

Speciate is organized around four pillars, each tagged with a delivery tier — **NOW** (active), **NEXT** (planned, scope TBD), **DREAM** (explicit north-star, not a schedule).

### 1. Prove Scale — `NOW`

The engine credibly handles huge populations and world sizes.

- Deterministic test framework
- Metrics framework + **live dashboard**
- Windows + Linux CI (which makes the status badges above live)

### 2. Prove Spectacle — `NOW`

GPU shaders, organic motion, and visual systems that **are** game mechanics — the "Golden Zone" applied to rendering, where an optimization is also the visual feature.

- WebGL shaders driven by simulation state
- Procedural organic motion
- See [docs/visuals/](docs/visuals/)

### 3. Play — `NEXT` (TBD)

Emergent gameplay layered on the proven engine, drawing from the biology and gameplay idea backlogs ([docs/biology/](docs/biology/), [docs/gameplay/](docs/gameplay/)).

### 4. Payoff — `NEXT` (TBD)

Career signal and R&D learning now; commercial paths stay open. The point today is demonstrable systems craft.

### Dreamland — `DREAM`

The explicitly-labeled north-star: Steam Early Access, a daughter-rescue narrative, taming, Drongos, and a Phase 2 MMO. **None of this is scheduled or promised** — it is aspirational framing for where the engine *could* go. See [docs/dreamland/](docs/dreamland/).

---

## Current Status

Honest, validated → target → stretch:

| Tier | Population | Platform | Status |
|------|-----------|----------|--------|
| **Validated** | 500K creatures | Linux | Actually tested and running |
| **Experimental** | 20K creatures | Windows | Runs, but **not officially supported**; root cause of the ceiling is unknown / under investigation |
| **Target** | 150K–200K | Cross-platform | The realistic near-term goal |
| **Stretch** | 1,000,000 | — | The "art of the possible" headline |

Linux is the supported, benchmarked platform today. Windows is a known-rough experimental path. The CI from Pillar 1 will turn these numbers into continuously-verified, live status.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                      ELECTRON / WEB SHELL                          │
├───────────────────────────────┬──────────────────────────────────┤
│  RUST BACKEND (Bevy ECS)       │  FRONTEND (PixiJS / WebGL)        │
│  in-process via NAPI-RS        │  Renderer                         │
│                                │                                   │
│  Simulation loop:              │  Render loop:                     │
│  • Perception (L0/L1 grid)     │  • Read positions (zero-copy)     │
│  • Behavior state machine      │  • Update sprites / shaders       │
│  • Steering (force accum.)     │  • Draw frame                     │
│  • Rayon-parallel movement     │                                   │
│  • Writes back buffer          │                                   │
│                                │                                   │
│  ZERO-COPY Float32Array  ──────┼──> Direct memory read,            │
│  via NAPI-RS double buffer     │    no serialization               │
└───────────────────────────────┴──────────────────────────────────┘
```

**This replaced the old stdio / MessagePack IPC entirely.** Any document still presenting stdio/MessagePack as the *current* transport is stale — the live path is zero-copy `Float32Array` over NAPI-RS.

**Backend (Rust + Bevy ECS):**
- **Rayon parallelization** — movement ~6.3x speedup, all cores engaged
- **Two-level spatial grid** — L0 (20m, fine perception) + L1 (60m, strategic classification) with early-exit on empty regions
- **Frequency throttling** — entity-ID bucketing with bitwise-AND, power-of-2 divisors only
- **Capability-marker ECS** — zero-sized-type markers for archetype stability (no archetype thrashing in hot loops)
- **Deterministic simulation** — reproducible runs, the foundation of the test framework

**Frontend (JS/web):**
- **PixiJS (WebGL)** rendering
- **Electron** desktop shell, with a **web-distribution** path

> ### Why data-oriented ECS wins — trading-grade latency engineering
>
> The author spent years chasing microseconds in trading platforms; Speciate applies the same discipline to a 500K-creature world. An **ECS lets us write data-oriented code, not object-oriented code** — and that is where the CPU edge comes from:
>
> - **"L3 is my RAM."** Components live in Bevy's archetype column tables (Struct-of-Arrays); the spatial grid adds a purpose-built contiguous SoA buffer on top — a single 32-byte proxy per entity, packed each tick by a zero-allocation parallel counting sort. The hot working set is engineered to stay **resident in cache**, not spill to DRAM.
> - **High IPC.** Contiguous storage keeps the execution units fed. Whole-tick IPC at population measures ~1.7; the isolated movement hot loop hits **~4.25** (Sprint 15 benchmark).
> - **Predictable branches.** Hot loops iterate uniform, homogeneous data → few mispredictions (measured **~0.77%** branch-miss rate). Zero-sized capability markers are added once at spawn and **never removed**, so archetypes stay stable — no layout thrash, no cache eviction churn.
> - **Fearless parallelism over already-contiguous data.** Rayon iterates the packed columns with no defensive locking, courtesy of Rust's data-race freedom.
>
> **The proof is measured, not asserted:** the engine is instrumented with real Linux `perf_event` hardware counters — CPU cycles, instructions, IPC, L1D/L1I/LLC cache misses, branch misses, and frontend/backend stalls — profiled the same way you'd profile a trading hot path (counters are Linux-only). See [docs/architecture/data-oriented-design.md](docs/architecture/data-oriented-design.md).

**See:** [docs/architecture/core-architectures.md](docs/architecture/core-architectures.md) (start here) · [docs/architecture/rust-js-thesis.md](docs/architecture/rust-js-thesis.md) (the showcase narrative) · [docs/architecture/electron-architecture.md](docs/architecture/electron-architecture.md)

---

## Getting Started

### Prerequisites

- **Rust** 1.75+ (`rustc --version`) — simulation backend
- **Node.js** 18+ (`node --version`) — Electron + frontend
- **npm** 10+ (`npm --version`)
- **System dependencies:**
  - **Linux** (the supported platform):
    - `perf` for performance metrics — `sudo apt-get install linux-tools-generic linux-tools-$(uname -r)`
    - Hardware-counter permissions for dev-tools:
      ```bash
      sudo sysctl -w kernel.perf_event_paranoid=1
      ```
      Run this if you see `Permission denied (os error 13)`. Verify with `cat /proc/sys/kernel/perf_event_paranoid` (should show 1 or lower). Persist with `echo "kernel.perf_event_paranoid=1" | sudo tee -a /etc/sysctl.conf`.
  - **macOS / Windows:** none (Electron bundles Chromium). Windows is experimental — see Current Status.

### Quick Start

```bash
cd apps/portal
npm run setup  # Install deps + build debug Rust + frontend (2-3 min)
npm run dev    # Launch app with hot reload
```

---

## Development Workflows

### Frontend changes (PixiJS / TypeScript) — instant feedback

```bash
npm run dev  # Start once, leave running
```

Edit any `.ts` / `.tsx` file → changes appear in under a second (Vite HMR).

### Rust changes (simulation) — fast iteration

```bash
npm run dev:rust   # Rebuild debug binary (~30 sec)
# Then restart Electron (Ctrl+R or relaunch npm run dev)
```

### Production build

```bash
npm run build          # Optimized Rust + frontend (3-5 min)
npm run package:linux  # Standalone .AppImage / .deb
```

Release builds are ~2x faster — use them for performance testing and installers.

---

## Command Reference

```bash
# First-time setup
npm run setup              # Install deps, build debug Rust, build frontend

# Development
npm run dev                # Vite dev server + Electron (hot reload)
npm run dev:vite           # Vite dev server only (browser testing)
npm run dev:electron       # Electron only (if Vite already running)
npm run dev:rust           # Rebuild debug Rust binary (~30 sec)

# Production builds
npm run build              # Optimized Rust + frontend
npm run build:rust         # Rust release build only (3-5 min)
npm run build:frontend     # Frontend build only

# Packaging
npm run package            # Build + package for current platform
npm run package:linux      # Linux .deb + .AppImage
npm run package:mac        # macOS .dmg
npm run package:win        # Windows .exe installer (experimental)

# Testing
npm test                   # Frontend tests
npm run type-check         # TypeScript validation
cd ../simulation && cargo test  # Rust tests
```

---

## Troubleshooting

**White screen on launch** — build the Rust binary: `npm run dev:rust` (debug) or `npm run build:rust` (release).

**"Cannot connect to Vite"** — make sure both processes run: `npm run dev` starts Vite + Electron in parallel.

**Slow Rust compilation** — use `npm run dev:rust` (~30 sec debug); reserve `npm run build:rust` (3-5 min) for production.

**No creatures rendering** — check the browser console, verify `apps/portal/dist` exists, rebuild with `npm run build`.

**Cargo build fails** — check `rustc --version` (need 1.75+); `cargo clean && cargo build --release`.

**npm install fails** — check `node --version` (need 18+); `npm cache clean --force && npm install`.

**"Permission denied (os error 13)" / hardware-counter errors** — `sudo sysctl -w kernel.perf_event_paranoid=1`. The simulation continues without hardware metrics (graceful fallback).

---

## Project Structure

```
/
├── apps/
│   ├── simulation/         # Rust / Bevy ECS engine
│   ├── portal/             # PixiJS frontend + Electron shell
│   │   ├── electron/       # Main process + preload
│   │   ├── src/            # TypeScript frontend (PixiJS)
│   │   └── dist/           # Vite build output
│   └── dev-ui/             # React developer tools (metrics, profiling) — never shipped
├── docs/
│   ├── ROADMAP.md          # The four pillars (NOW / NEXT / DREAM)
│   ├── architecture/       # Engine architecture + rust-js-thesis
│   ├── scale/              # Pillar 1: metrics, dashboard, test framework, CI
│   ├── visuals/            # Pillar 2: shaders and visual systems
│   ├── biology/            # DNA design + Pillar 3 idea backlog
│   ├── gameplay/           # Pillar 3 gameplay idea backlog
│   ├── dreamland/          # Aspirational: Steam, narrative, MMO, strategy
│   └── archive/            # ADRs — what was tried and abandoned
└── .claude/                # AI agent definitions, commands, hooks
```

---

## Key Design Principles

### DNA-Driven Design

The DNA is the creature; everything else is expression. All traits flow from genes (size, speed, perception, aggression), complex behaviors emerge from primitives, and every advantage carries a cost. See [docs/biology/ideas/dna-driven-design.md](docs/biology/ideas/dna-driven-design.md).

### The Golden Zone

The best optimization *is* the feature. When skipping work matches real biology (giants ignore mice, frozen prey is camouflaged, satiated predators rest) or makes a rendering trick into a visual mechanic, performance and gameplay win at once.

### Test-Driven Development

Red-Green-Refactor for all changes; deterministic simulation makes behavior reproducible and testable. See [CLAUDE.md](CLAUDE.md).

### Emergence Over Scripting

Systems, not scripts. Sociality emerges from personal space + flocking + low aggression — it is never a single "sociality" gene.

---

## Resources

- [docs/ROADMAP.md](docs/ROADMAP.md) — the four pillars and tiers
- [docs/architecture/rust-js-thesis.md](docs/architecture/rust-js-thesis.md) — the Rust × JS showcase narrative
- [docs/architecture/core-architectures.md](docs/architecture/core-architectures.md) — all core architectural principles, indexed
- [CLAUDE.md](CLAUDE.md) — TDD and DNA enforcement
- [Bevy ECS](https://bevyengine.org/) · [Pixi.js 8.x](https://pixijs.com/8.x/guides) · [napi-rs](https://napi.rs/) · [Rust Book](https://doc.rust-lang.org/book/)

---

## License

[TODO: Add license]

---

**The DNA is the creature. Everything else is just expression.**
