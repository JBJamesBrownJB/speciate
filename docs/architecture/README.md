# 📖 Architecture — Reference

> **Category: 📖 REFERENCE.** This folder is *stable knowledge*, not a feature
> lifecycle. Nothing here is an idea, a backlog item, or a "done" feature log —
> it is the standing explanation of how the engine is built and why. These docs
> change when the architecture changes, not when a sprint ships a feature.
>
> For the documentation category legend and the reference-vs-lifecycle split,
> see [`../documentation-standards.md`](../documentation-standards.md). For what is being built **now**, see
> [`../ROADMAP.md`](../ROADMAP.md).

**Legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND

---

## Purpose

The canonical, claim-checked description of Speciate's engine architecture.
Speciate is a **portfolio showcase** read by hiring engineers, so every
technical claim in these docs is traceable to code under `apps/`. Read
[`core-architectures.md`](./core-architectures.md) first; it indexes every
principle the rest of the codebase must align with.

---

## Documents

| Document | What it covers |
|----------|----------------|
| [`core-architectures.md`](./core-architectures.md) | **Start here.** Index of the foundational principles every feature must align with: DNA-driven design, force accumulation, the two-level spatial grid (L0 20m / L1 60m), capability-marker ECS, frequency throttling, and binary IPC. |
| [`rust-js-thesis.md`](./rust-js-thesis.md) | The showcase argument: why a hybrid Rust-core / web-frontend split is the right shape, and how the seam between them is made nearly free via zero-copy NAPI `Float32Array` IPC. Written for a technical reader skimming for signal. |
| [`ecs-optimization-playbook.md`](./ecs-optimization-playbook.md) | Data-Oriented Design in practice for Bevy ECS: cache locality, archetype stability, hot/cold data separation, and the Rayon movement parallelization (~6.3x across all cores) behind the scale numbers. This is also where the project's data-oriented-design rationale lives. |
| [`electron-architecture.md`](./electron-architecture.md) | The Electron desktop shell and the NAPI-RS zero-copy shared-memory bridge between the Rust simulation and the TypeScript/PixiJS frontend (replaces the archived stdio/MessagePack path). |
| [`behavior-engine.md`](./behavior-engine.md) | Reynolds steering behaviors on Bevy ECS: the force-accumulation pattern, per-behavior steering systems, creature state machines, and system ordering. |
| [`snapshot-interpolation.md`](./snapshot-interpolation.md) | Smooth motion from the 20 Hz sim across the NAPI seam: render ~1 tick in the past, drive α from a playout clock (never reset on arrival), pool snapshots into SoA slots. The Valve/Bernier + Fiedler "render in the past" technique applied locally. |

> **Note on `data-oriented-design`:** there is no standalone
> `data-oriented-design.md` in this folder. The DOD material is folded into
> [`ecs-optimization-playbook.md`](./ecs-optimization-playbook.md) and
> [`core-architectures.md`](./core-architectures.md). The project glossary lives
> at the repository root (`GLOSSARY.md`), not here.

### Assets

| Folder | What it is |
|--------|------------|
| `diagrams/` | 📖 Architecture diagram image(s). Assets, not prose — referenced by the docs above. |

---

## Related

- 🚧 [`../scale/`](../scale/) — Pillar 1 (NOW): how the architecture is being proven at population.
- 📖 [`../archive/`](../archive/) — architectures that were tried and abandoned (ADR-style "what we tried and why we stopped").
- 📖 [`../protocol/`](../protocol/) — stable wire contracts (e.g. the behavior enum).
