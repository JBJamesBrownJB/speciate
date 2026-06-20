# Speciate Documentation — Map & Legend

**Speciate — a million-creature artificial-life engine, where Rust's fearless parallelism meets the web's visual playground.**

This is the authoritative **map** of the `docs/` tree. It is a **portfolio showcase** demonstrating Rust × JS systems craft: a Rust + Bevy ECS backend driving large creature populations, rendered through a PixiJS/WebGL frontend over a zero-copy NAPI-RS seam.

For overall direction (the four pillars and NOW/NEXT/DREAM tiers), start with the **[Roadmap](./ROADMAP.md)**. This page tells you, at a glance, **what kind** of document every folder holds.

---

## Legend — the KIND of each doc

| | Category | Meaning |
|--|----------|---------|
| 📖 | **Reference** | Stable knowledge. NOT a feature lifecycle. Architecture, contracts, glossaries, retrospectives, archived decisions, evidence/assets. |
| 💡 | **Ideas** | Brainstormed, exploratory, **NOT committed**. Lives in `*/ideas/` and freeform concept folders. |
| 🚧 | **In progress (NOW)** | Being built right now — the active NOW pillars. Cross-links the [Roadmap](./ROADMAP.md) NOW tier. |
| 📋 | **Planned** | Approved/designed but **not started**. Lives in `*/todo/` and deferred plans. |
| ✅ | **Done** | Implemented and working. Lives in `*/done/`. |
| 🌙 | **Dreamland** | Aspirational north-star. **Not scheduled.** |

**Reference (📖) is separated from feature-lifecycle docs.** Lifecycle areas use the `ideas/` 💡 → `todo/` 📋 → `done/` ✅ progression. Reference areas (architecture, protocol, process, incidents, archive, glossary) do **not** — they are stable knowledge, not features moving through a pipeline. This distinction is codified in [docs/documentation-standards.md](./documentation-standards.md).

---

## Scale Status (the honest ladder)

[![Target](https://img.shields.io/badge/target-1M_creatures-blue)](./ROADMAP.md)
[![Linux](https://img.shields.io/badge/Linux-500K_validated-brightgreen)](./scale/README.md)
[![Windows](https://img.shields.io/badge/Windows-20K_experimental-orange)](./scale/README.md)

- **Stretch / "art of the possible":** 1,000,000 creatures. Target headline, not yet achieved.
- **Validated (Linux):** 500,000 creatures. Actually tested.
- **Experimental (Windows):** 20,000 creatures. Not officially supported; gap under investigation.

> Badges are **static placeholders**. Pillar 1's CI is what will make them live.

---

## Start Here

- **[Roadmap](./ROADMAP.md)** — 📖 the four pillars with NOW / NEXT / DREAM tiers. The source of truth for what is being built NOW.
- **[Rust × JS Thesis](./architecture/rust-js-thesis.md)** — 📖 the showcase narrative: why Rust backend, why the web frontend, why the zero-copy NAPI seam makes the hybrid work.
- **[Core Architectures](./architecture/core-architectures.md)** — 📖 index of all core architectural principles. Read before adding any feature.

---

## The Whole Tree, by Category

### 📖 Reference — stable knowledge (NOT a lifecycle)

| Folder / file | Purpose |
|---------------|---------|
| [ROADMAP.md](./ROADMAP.md) | Authoritative NOW / NEXT / DREAM tiers. Every 🚧 banner cross-links here. |
| [documentation-standards.md](./documentation-standards.md) | Documentation standards — including this taxonomy and the reference-vs-lifecycle split. |
| [architecture/](./architecture/) | Engine architecture & design: Rust×JS thesis, core architectures, ECS optimization playbook, behavior engine, Electron/NAPI IPC, diagrams. |
| [protocol/](./protocol/) | Stable cross-boundary contracts (e.g. the Behavior Enum contract). |
| [process/](./process/) | Engineering retrospectives and production-incident lessons. |
| [incidents/](./incidents/) | Post-mortem records (crash fixes, root-cause history). |
| [archive/](./archive/) | Architecture Decision Records — what was tried and abandoned, kept for learning. |
| `../GLOSSARY.md` | Project glossary (lives at **repo root**, not under `docs/`). |

> Evidence/asset folders are **data, not prose**: `performance/snapshots/` (JSON benchmark runs), `performance/history/` (PNG perf snapshots), `architecture/diagrams/` (PNG). Treat a benchmark JSON as evidence, not a "doc."

### 🚧 In progress (NOW) — the active pillars

| Folder | Pillar | Purpose |
|--------|--------|---------|
| [scale/](./scale/) | 1 — Prove Scale | Metrics framework, live dashboard, deterministic test framework, cross-OS/CI, the march toward 1M. |
| [visuals/](./visuals/) | 2 — Prove Spectacle | GPU shaders and organic motion that double as game mechanics (Golden Zone). |

> A few `*/todo/` Golden-Zone items (`biology/todo/motion-detection.md`, `biology/todo/hunger-gating.md`) sit at the Pillar 2/3 seam per the [Roadmap](./ROADMAP.md). They are **not yet built**, so their honest base label is 📋 Planned.

### Lifecycle areas (Pillar 3 fodder) — mixed 💡 / 📋 / ✅

Each uses the `ideas/` 💡 → `todo/` 📋 → `done/` ✅ progression. See each area's `README.md`.

| Folder | Purpose |
|--------|---------|
| [biology/](./biology/) | DNA-driven traits & behaviors. `done/` ✅ (perception, seeking, FOV, movement physics…), `ideas/` 💡, `todo/` 📋, plus `biology-notes.md` (zoologist log). |
| [gameplay/](./gameplay/) | Player-facing mechanics & UI. `done/` ✅ (minimap), `ideas/` 💡 (taming, territory, trophy hunting, interactions). |
| [performance/](./performance/) | Optimization lifecycle. `done/` ✅ (Rayon, spatial grid, viewport indexing), `ideas/` 💡 (SIMD, GPU-compute, LOD), `todo/` 📋, plus `history/` + `snapshots/` 📖 evidence. Distinct from `scale/`: performance = optimization lifecycle, scale = Pillar 1 NOW deliverables. |
| [testing/](./testing/) | Test infrastructure. `done/` ✅ (specification framework, ghost crits), `ideas/` 💡, `bugs/` (active notes), plus deferred dev-tools/hot-load 📋 notes. |

### 💡 Ideas — exploratory, not committed

| Folder | Purpose |
|--------|---------|
| [terrain/](./terrain/) | Terrain concept notes (cellular-automata terrain, habitat hints). Not implemented. |
| [research/](./research/) | Exploratory research (e.g. agent-id / nanoid). Future enhancement, not committed. |
| `*/ideas/` | The idea backlogs inside biology, gameplay, performance, testing, visuals. |

### 📋 Planned — approved/designed, not started

| Folder | Purpose |
|--------|---------|
| [lod-ai-framework/](./lod-ai-framework/) | `PLAN.md` — design complete, implementation deferred. |
| `*/todo/` | Approved-but-unstarted items inside lifecycle areas. |

### 🌙 Dreamland — aspirational north-star (not scheduled)

| Folder | Purpose |
|--------|---------|
| [dreamland/](./dreamland/) | Steam Early Access, the daughter-rescue narrative, taming, Drongos, Phase 2 MMO, business strategy. Labeled aspirational throughout. |

---

## Engine Facts (verified against `apps/`)

- **Rust + Bevy ECS** backend (`apps/simulation/`).
- **Rayon parallelization** — movement integration ~6.3× speedup, all cores (`simulation/movement/systems.rs`).
- **Zero-copy Float32Array IPC via NAPI-RS** — replaced the old stdio/MessagePack path.
- **Two-level spatial grid** — L0 20m / L1 60m (`CELL_SIZE = 20.0`, `L1_CELL_SIZE = CELL_SIZE * 3.0` in `simulation/spatial/constants.rs`).
- **Frequency throttling** — power-of-2 bucketing spreads expensive per-creature work across ticks.
- **Capability-marker ECS** — zero-sized markers added once at spawn keep archetypes stable.
- Frontend is **PixiJS (WebGL)** on Electron, with a web-distribution path open.

---

## Documentation Standards

When creating or updating docs, follow [docs/documentation-standards.md](./documentation-standards.md):

- Documentation describes **WHAT and WHY, not HOW** — code is the source of truth for implementation.
- Reference `file:line` instead of duplicating code blocks.
- Use **kebab-case** file names and concise, high-level prose.
- **Apply the taxonomy above:** put each new doc in the folder whose category matches its KIND, and keep reference separate from feature lifecycle.
- **Honesty mandate:** engineers read this. Never overclaim. Show a validated → target → stretch ladder, and verify any technical claim against the code before asserting it.
