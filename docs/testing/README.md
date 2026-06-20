# 🧪 Testing

> **Category legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
> Full map + legend: [`docs/README.md`](../README.md) · What's actively NOW: [`docs/ROADMAP.md`](../ROADMAP.md)

**Purpose:** The test-infrastructure lifecycle — the tooling that keeps the simulation honest. Covers the spec-driven "source of truth" framework (TOML specs in `apps/simulation/specs/`, headless `cargo test` + visual `cargo run` verification), debug-visualization aids, and the running log of behavioral bugs under investigation.

**This is a feature-lifecycle area.** Documents move through `ideas/` → `todo/` → `done/`:

| Subfolder | Category | Meaning |
|-----------|----------|---------|
| [`ideas/`](ideas/) | 💡 IDEAS | Brainstormed test/debug tooling, **not committed**. `force-visualisation.md`. |
| [`done/`](done/) | ✅ DONE | **Implemented & working.** `specification-framework.md` (the spec-driven test architecture), `ghost-crits.md` (deterministic test fixtures). |

Also at this level:

- [`bugs/`](bugs/) — 📖 active investigation notes for live defects, not lifecycle features. `jitter-high-populations.md` (jerky movement at high population — ✅ resolved via push-on-swap + snapshot interpolation), `f32-id-precision-ceiling.md` (🔴 critical, deferred — creature ids lose precision above ~16.7M cumulative spawns), `zipping-crits.md` (creatures breaching force caps via emergency-force path).
- `dev-tools-toggle-features.md` — 📋 deferred design for a Dev-UI debug control plane (toggle FOV / forces / spatial-grid layers).
- `hot-load-config.md` — 📋 deferred hot-reload config system (runtime constant tuning without recompilation).

**What's in progress NOW?** Testing is infrastructure, not a NOW pillar — it underwrites the active work. See [`docs/ROADMAP.md`](../ROADMAP.md) for the authoritative NOW / NEXT / DREAM tiers.
