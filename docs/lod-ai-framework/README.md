# 🎚️ LOD AI Framework

> **Category legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
> Full map + legend: [`docs/README.md`](../README.md) · What's actively NOW: [`docs/ROADMAP.md`](../ROADMAP.md)

**Category:** 📋 **Planned** — design complete, **implementation deferred**. Approved direction with a full design doc, but not yet built.

**Purpose:** A Level-of-Detail AI framework that scales compute by visibility — every entity carries an `Lod` enum and each system branches on it — to push toward 150K+ creatures within the ~45ms tick budget.

**What's inside:**

| File | What it covers |
|------|----------------|
| [`PLAN.md`](PLAN.md) | The full design: three LOD levels (full / reduced / minimal fidelity), the per-tick `update_lod_system` decision logic, LOD-tax cost analysis, why a component (not markers, to avoid archetype thrashing), phased implementation tasks, and target files to create/modify. |

**Background:** Originated as Sprint 16 (`feat/sprint-16-lod-ai-framework`); foundation work (cell-culling fix, force-multiplier refactor, constants audit) shipped, but the LOD implementation itself was deferred. The first proposed step is a perception-sorting benchmark gating the rest (see `apps/simulation/src/simulation/perception/systems.rs`).

**Status NOW?** Not in active development. The NOW pillars are [`docs/scale/`](../scale/) and [`docs/visuals/`](../visuals/) — see [`docs/ROADMAP.md`](../ROADMAP.md) for the authoritative NOW / NEXT / DREAM tiers. Per the honesty mandate, this stays 📋 Planned until built.
