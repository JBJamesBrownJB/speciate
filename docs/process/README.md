# 🛠️ Process

> **Category legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
> Full map + legend: [`docs/README.md`](../README.md) · What's actively NOW: [`docs/ROADMAP.md`](../ROADMAP.md)

**Category: 📖 Reference.** Engineering retrospectives and production-incident lessons — the hard-won knowledge about *how* we build, distilled from things that broke. Not a feature lifecycle; these are durable takeaways meant to stop the same class of failure twice.

**What's inside:**

| Document | What it covers |
|----------|----------------|
| [`lessons-learned.md`](lessons-learned.md) | Post-incident retrospectives with root causes, testing/process gaps, and prevention checklists. Example: the save-state corruption at scale (>10K creatures) tracing back to MessagePack limits and a worker-thread shutdown race. |

**Why it lives here:** Bugs are forgivable; repeating them is not. This area captures the methodology gaps behind incidents — test at production scale, synchronize async work, verify the Rust→NAPI→Electron seam — so the lessons outlive the sprint that learned them. For the blow-by-blow post-mortems themselves, see [`docs/incidents/`](../incidents/README.md).
