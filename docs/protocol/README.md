# 🔌 Protocol

> **Category legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
> Full map + legend: [`docs/README.md`](../README.md) · What's actively NOW: [`docs/ROADMAP.md`](../ROADMAP.md)

**Category: 📖 Reference.** Stable cross-boundary contracts — the agreed wire formats and shared enums that keep the Rust simulation and the TypeScript frontend in lockstep across the NAPI/IPC seam. Not a feature lifecycle; these are living specifications that both sides must honor.

**What's inside:**

| Document | What it covers |
|----------|----------------|
| [`behaviors.md`](behaviors.md) | The `BehaviorMode` enum contract — discriminant values shared by Rust (`apps/simulation/src/simulation/creatures/components/state.rs`) and TS (`apps/portal/src/types/behaviors.ts`), plus the append-only rules for evolving it safely. |

**Why it lives here:** A mismatched discriminant silently mislabels every creature on screen. These contracts exist so a change on one side is impossible to forget on the other — versioned, append-only, and authoritative for both languages.
