# 🚨 Incidents

> **Category legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
> Full map + legend: [`docs/README.md`](../README.md) · What's actively NOW: [`docs/ROADMAP.md`](../ROADMAP.md)

**Category: 📖 Reference.** Post-mortem records — crash fixes and root-cause history for specific production failures. Not a feature lifecycle; this is an append-only archive of what broke, why, and how it was resolved.

**What's inside:**

| Document | What it covers |
|----------|----------------|
| [`history/CRASH_FIX_SUMMARY.md`](history/CRASH_FIX_SUMMARY.md) | The `DoubleBuffer` race condition — random `SIGTRAP` crashes under rapid large-trial spawning, root-caused to unsafe raw-pointer buffer swapping (`apps/simulation/src/ipc/bridge/double_buffer.rs`) and fixed with a safe `Vec`-swap, verified by a 375K-creature stress test. |

**Why it lives here:** A post-mortem is evidence — a permanent record of a single failure's anatomy. The generalized takeaways drawn from these incidents are distilled in [`docs/process/`](../process/README.md); this area keeps the source records intact.
