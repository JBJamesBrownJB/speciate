# docs/ — Documentation Area Guide

See [`../AGENTS.md`](../AGENTS.md) (and root [`/AGENTS.md`](../AGENTS.md)) for global rules. This file holds only `docs/`-local specifics: where things live and the stale traps to reject.

- **Map of the whole tree (categories, what every folder holds):** [`README.md`](./README.md).
- **How to write a doc (WHAT/WHY-not-HOW, taxonomy legend, good-vs-bad + `done/todo/ideas` templates):** [`documentation-standards.md`](./documentation-standards.md).

---

## Verified Reference Points (reject stale versions)

These are the doc-authoring landmines — signals that go stale and get mis-cited. Each is verified against the code; do not contradict them.

- IPC / Electron seam lives in [`architecture/electron-architecture.md`](./architecture/electron-architecture.md). The old `napi-architecture.md` link is **dead** — do not cite it.
- IPC is **zero-copy `Float32Array` via NAPI-RS double buffer**. The old stdio/MessagePack path is archived under `archive/`, **not current**.
- Spatial grid is **L0 20m / L1 60m** — not 10m/30m (`CELL_SIZE = 20.0`, `L1_CELL_SIZE = CELL_SIZE * 3.0` in `apps/simulation/src/simulation/spatial/constants.rs:1`).
- Direction lives in [`ROADMAP.md`](./ROADMAP.md) pillars; point-in-time history lives in `sprint_summaries/`. There is no "current sprint" framing.
