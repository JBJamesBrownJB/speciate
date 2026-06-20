# ⚡ Performance

> **Category legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
> Full map + legend: [`docs/README.md`](../README.md) · What's actively NOW: [`docs/ROADMAP.md`](../ROADMAP.md)

**Purpose:** The optimization lifecycle — *how* we make the engine fast. Documents the techniques that let the Rust simulation push toward the 150K–200K creature target: Rayon parallelization, the hierarchical spatial grid, viewport-spatial indexing, and the backlog of bigger swings (SIMD, GPU compute, LOD).

**Distinct from [`docs/scale/`](../scale/README.md).** Keep these straight:

- **`performance/` = the optimization lifecycle** — the catalogue of techniques and their lifecycle stage (idea → planned → done). The *how-we-make-it-fast* knowledge base.
- **`scale/` = the Pillar 1 NOW deliverables** — the active 🚧 work that *proves* the engine scales. Scale consumes the techniques documented here; this area is where they live as durable feature docs.

**This is a feature-lifecycle area.** Documents move through `ideas/` → `todo/` → `done/`:

| Subfolder | Category | Meaning |
|-----------|----------|---------|
| [`ideas/`](ideas/) | 💡 IDEAS | Brainstormed optimizations, **not committed**. SIMD, GPU compute, LOD rendering, memory layout, zero-copy serialization, object pooling, and more. |
| [`todo/`](todo/) | 📋 PLANNED | Approved direction, **not yet started**. Dual spatial grid, flat 2D array, dynamic cell sizing, cache-metrics instrumentation, biomass-grid minimap. |
| [`done/`](done/) | ✅ DONE | **Implemented & working** in `apps/simulation/`. `rayon-parallelization.md`, `hierarchical-spatial-grid.md`, `viewport-spatial-indexing.md`, `fuse-steering.md`, `system-update-frequency.md`, `buffer-transfer-baseline.md`. |

Also at this level — 📖 **reference data, not prose** (never treat a benchmark JSON or a chart PNG as a "doc"):

- [`snapshots/`](snapshots/) — captured benchmark runs (`*.json`), e.g. `200k_randomDNA_*`, `360k_mixed_density_*`. Empirical evidence of where the engine sits at scale.
- [`history/`](history/) — performance chart images (`perf_*.png`) tracking improvement over time.

**What's in progress NOW?** The optimizations land in service of the NOW pillars — see [`docs/scale/`](../scale/README.md) (Prove Scale) and [`docs/ROADMAP.md`](../ROADMAP.md) for the authoritative NOW / NEXT / DREAM tiers.
