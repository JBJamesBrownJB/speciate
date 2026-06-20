# 🔬 Research

> **Category legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
> Full map + legend: [`docs/README.md`](../README.md) · What's actively NOW: [`docs/ROADMAP.md`](../ROADMAP.md)

**Category:** 💡 **Ideas** — exploratory investigations and decision records for **future enhancements**, **not committed**. These weigh options for work that may or may not happen post-MVP.

**Purpose:** Capture trade-off analysis for forward-looking technical choices so the decision context survives even if the change is deferred.

**What's inside:**

| File | What it explores |
|------|------------------|
| [`agent-id-nanoid.md`](agent-id-nanoid.md) | Whether to replace the current monotonic `u32` agent ID (see `apps/simulation/src/simulation/systems.rs`) with globally-unique IDs (NanoId vs Snowflake vs UUIDv7) for multi-instance scenarios. Records the post-MVP migration path and memory/serialization costs. |

**Status NOW?** Nothing here is scheduled — these are backlog research notes. The active NOW pillars are [`docs/scale/`](../scale/) and [`docs/visuals/`](../visuals/) — see [`docs/ROADMAP.md`](../ROADMAP.md) for the authoritative NOW / NEXT / DREAM tiers.
