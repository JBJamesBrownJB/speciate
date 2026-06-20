# AGENTS.md — Speciate (root)

> **A million-creature artificial-life engine, where Rust's fearless parallelism meets the web's visual playground.**

This is the **global** agent guide for the Speciate repo. It applies to **any AI agent or human contributor**. Nested `AGENTS.md` files (closest-file-wins) hold area-specific rules — when working in a subtree, read that file too; it extends, not replaces, this one.

> Claude Code users also have specialized agents and slash commands under `.claude/` — those are an optional extra, never a requirement for this guide.

---

## What this is

Speciate is a **portfolio showcase**: a high-performance artificial-life engine plus visual sandbox demonstrating **Rust × JS systems craft**. Audience: hiring engineers and the author. It is **not a game in production**. Creatures are DNA-driven — complex behaviour **emerges** from genetic primitives, no scripted NPCs. The headline is the **engineering** (scale, determinism, architecture).

### Four pillars (+ Dreamland) — see `docs/ROADMAP.md`
Tiers: **NOW** (active) · **NEXT** (intended, scope TBD) · **DREAM** (north-star, not scheduled).

1. **Prove Scale** — `NOW` — credibly handle huge populations, provably, cross-OS. Home: `docs/scale/`.
2. **Prove Spectacle** — `NOW` — GPU shaders + organic motion that *are* game mechanics. Home: `docs/visuals/`.
3. **Play** — `NEXT (TBD)` — emergent gameplay on the proven engine. Draws from `docs/biology/`, `docs/gameplay/`.
4. **Payoff** — `NEXT (TBD)` — career signal + R&D; commercial paths open, nothing scheduled.
- **Dreamland** — `DREAM` — Steam EA, narrative, taming, Phase 2 MMO. Aspirational. Home: `docs/dreamland/`.

Build order: prove engine (1+2) → layer play (3) → realize payoff (4) → chase dream.

### Honest scale ladder (validated → target → stretch)
| Tier | Population | Platform | Status |
|------|-----------|----------|--------|
| Stretch ("art of the possible") | 1,000,000 | — | Headline target, **not yet achieved** |
| Validated | 500,000 | Linux | Tested; the supported, benchmarked platform |
| Experimental | 20,000 | Windows | Runs, **not officially supported**; ceiling under investigation |
| Target (near-term) | 150K–200K | Cross-platform | Realistic near-term goal |

README/docs scale badges are static placeholders; Pillar 1's CI is what makes them live.

### The thesis (see `docs/architecture/rust-js-thesis.md` + `docs/architecture/data-oriented-design.md`)
- **Right shape:** a Rust core + web frontend joined by a **zero-copy NAPI `Float32Array` seam** — Rust throughput *and* the web's visual/distribution reach, with the serialization tax that sinks most hybrids made nearly free.
- **Fearless parallelism:** the borrow checker turns data races into compile errors, so the hot loop parallelizes with Rayon without defensive locking.
- **Type system as guardrail:** ownership / lifetimes / `Send`+`Sync` make the compiler a tireless reviewer — the reason **AI-agent-authored systems code** is safe to ship.
- **Data-oriented ECS = trading-grade latency engineering:** SoA archetype columns keep the hot working set cache-resident, proven with real Linux `perf_event` counters.

---

## Repo map

```
apps/
  simulation/   Rust / Bevy ECS engine (NAPI addon)   → apps/simulation/AGENTS.md
  portal/       PixiJS frontend + Electron shell        → apps/portal/AGENTS.md
  dev-ui/       React developer tools (metrics/profiling) — never shipped
docs/           ROADMAP, architecture, lifecycle areas   → docs/AGENTS.md
sprint_summaries/  point-in-time sprint history
.claude/        Claude-Code-specific agents/commands/hooks (optional extra)
```

There is no root-level `package.json` aggregator — all build/test commands run inside an `apps/*` subtree.

---

## Global guardrails (hard rules)

These are **conventions enforced by human discipline and real test commands**, not by live tooling. (`.claude/hooks/` exist but are currently unregistered, so treat nothing here as auto-enforced.) They are still mandatory.

- **TDD red-green-refactor.** Write a failing test first, make it pass minimally, then refactor. Run the full suite before and after every change. Real infra: `cargo test` (Rust), `vitest` (TS).
- **DNA-driven design.** Creature traits live in DNA (`apps/simulation/src/simulation/creatures/dna/`); complex behaviour emerges. No hardcoded magic-number traits as the end state. The DNA system is **real but nascent** (2 genes: `size_gene`, `fov_gene` + `express_gene()`); other traits are still hardcoded with migration TODOs — that's ongoing, not done. When adding or changing a creature trait, log the decision and its realistic ranges/trade-offs in `docs/biology/biology-notes.md`.
- **Portal vs Dev-UI separation.** `apps/portal` is the player-facing game (PixiJS); `apps/dev-ui` is developer metrics/profiling (React, never shipped). Never put dev metrics in portal, never put gameplay UI in dev-ui. "Would a player see this?" YES → portal, NO → dev-ui.
- **Binary IPC.** High-frequency Rust↔TS data uses the **zero-copy `Float32Array` via NAPI-RS double buffer** — never JSON on the hot path. JSON only for low-frequency config/save. (The old stdio/MessagePack path is **archived**, not current.)
- **Documentation standards.** Describe **WHAT and WHY, not HOW**; comments are a code smell (reference `file:line` instead of duplicating code). Follow the doc taxonomy (📖 Reference / 💡 Ideas / 🚧 In-progress-NOW / 📋 Planned / ✅ Done / 🌙 Dreamland). See `docs/AGENTS.md`.
- **Code quality.** No `console.log` (use `console.error` for real errors only); avoid `any` in TS (`tsc --noEmit` is the type gate); PixiJS interactions use its event system, not raw DOM.
- **Spec approval.** `apps/simulation/specs/` are specification tests — changing them requires **explicit human approval**.

### Architecture facts (use these; reject stale versions)
- **Backend:** Rust + Bevy ECS. **IPC:** zero-copy NAPI `Float32Array` double buffer.
- **Tick rate: 20 Hz** — `apps/simulation/src/napi_addon/simulation_engine.rs:39` (`TARGET_SIMULATION_HZ = 20.0`), single-tick. (Dual-tick / 22.2 Hz is abandoned/archived.)
- **Two-level spatial grid: L0 = 20 m / L1 = 60 m** — `apps/simulation/src/simulation/spatial/constants.rs` (`CELL_SIZE = 20.0`, `L1_CELL_SIZE = CELL_SIZE * 3.0`). (Not 10 m / 30 m.)
- **Patterns:** force accumulation (`accel += force`), capability-marker ECS (ZST markers added at spawn, never removed), frequency throttling (power-of-2 bucketing), Rayon movement parallelization (manual `Vec` collect → `par_iter_mut()`).
- IPC/desktop architecture doc: `docs/architecture/electron-architecture.md` (there is no `napi-architecture.md`).

### Golden Zone
Actively seek optimizations that **are** the biological feature (e.g. skip perception of small entities → giants ignore mice; satiated predators skip prey detection → post-meal rest). Performance win + gameplay win = double value. Every advantage must have a cost built into physics/biology.

---

## Top-level commands

All commands are verified against `apps/*/package.json` and `Cargo.toml`. Run them inside the named subtree.

### First-time setup (`apps/portal`)
```bash
npm run setup          # install + build debug Rust addon + build frontend
```

### Develop (`apps/portal`)
```bash
npm run dev            # parallel Vite + Electron (debug Rust, hot reload)
npm run dev:release    # rebuild release Rust addon, then Vite + Electron
npm run dev:tools      # Electron with the dev-ui window (--dev-tools)
npm run dev:rust       # rebuild debug NAPI addon only
```
Dev-UI window standalone (`apps/dev-ui`): `npm run dev`.

### Build (`apps/portal`)
```bash
npm run build          # build:rust (release NAPI addon) + build:frontend (tsc && vite build)
```

### Package (`apps/portal`)
```bash
npm run package        # current platform; also package:win / package:mac / package:linux
```

### Test
```bash
# Frontend (apps/portal)
npm test               # vitest
npm run type-check     # tsc --noEmit

# Backend (apps/simulation)
cargo test                              # default features
cargo build --features dev-tools        # dev build (instrumentation; perf_event is Linux-only)
cargo build --no-default-features       # verify it compiles without instrumentation
```
The Rust NAPI addon is built via napi-rs (`napi build`), exposed as `npm run build` / `npm run build:debug` in `apps/simulation` — not bare `cargo build`. See `apps/simulation/AGENTS.md` for the full Cargo/spec command set.

---

**Working in a subtree? Read that area's `AGENTS.md`:** `apps/simulation/AGENTS.md`, `apps/portal/AGENTS.md`, `docs/AGENTS.md`.
