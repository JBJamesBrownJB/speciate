# Speciate Roadmap

> **Speciate** — a million-creature artificial-life engine, where Rust's fearless
> parallelism meets the web's visual playground.

This roadmap is organized around **four pillars** and a labeled **Dreamland**
north-star. Each pillar carries tiers:

- **NOW** — actively being built / next up. Concrete, committed deliverables.
- **NEXT** — intended direction, scope still TBD. Honest about uncertainty.
- **DREAM** — aspirational north-star. **Not a schedule.** See [Dreamland](#dreamland).

**Honesty mandate:** engineers read this. Numbers below are split into a
**validated → target → stretch** ladder. Anything not yet proven is labeled as a
target or experiment, never as a shipped capability.

---

## Scale Status (the honest ladder)

| Tier | Population | Status |
|------|-----------|--------|
| **Stretch / "art of the possible"** | 1,000,000 creatures | Target headline. Not yet achieved. |
| **Validated (Linux)** | 500,000 creatures | Actually tested. |
| **Experimental (Windows)** | 20,000 creatures | Not officially supported. Root cause of the gap is unknown and under investigation. |

The README's status badges (shields.io static placeholders) reflect this exact
ladder. They are placeholders today; **Pillar 1's CI is what will make them live.**

---

## Pillar 1 — PROVE SCALE  ·  *tier: NOW*

**Thesis:** the engine credibly handles huge populations and world sizes, and we
can *prove* it on demand, on more than one OS.

### Engine foundations (validated, in `apps/simulation/`)

- **Rust + Bevy ECS** backend.
- **Rayon parallelization** — movement integration ~6.3x speedup, engaging all
  cores (`simulation/movement/systems.rs`).
- **Two-level spatial grid** — L0 fine cells and L1 coarse cells (3×3 L0 per L1);
  see `simulation/spatial/constants.rs` for the authoritative cell sizes.
- **Frequency throttling** — power-of-2 bucketing spreads expensive per-creature
  work across ticks.
- **Capability-marker ECS** — zero-sized capability markers added once at spawn
  keep archetypes stable (no archetype thrash in hot loops).
- **Deterministic simulation** — same seed, same trajectory; the basis for the
  test framework below.
- **Simulation tick rate** — see `napi_addon/simulation_engine.rs`
  (`TARGET_SIMULATION_HZ`) for the authoritative value.

### NOW deliverables

- **Deterministic test framework** — seed-replayable scenarios that assert
  trajectory/state invariants, giving us a regression net at scale.
- **Metrics framework + LIVE DASHBOARD** — per-system timings, ECS/archetype
  health, and hardware counters surfaced in **dev-ui** (developers only — never
  Portal). The dashboard turns "it's fast" into observable, defensible numbers.
- **Windows + Linux CI** — cross-OS build + benchmark pipeline. This is what
  promotes the README scale badges from static placeholders to live status.
- **The 500K-Linux / 20K-Windows reality** — keep it honest in docs and badges.
  The Windows ceiling is an open **investigation**, not a hidden failure:
  document findings as the root cause is isolated.
- **March toward 1M** — the headline stretch target. Each optimization is
  measured against the validated 500K baseline, not asserted.

**Pillar 1 home:** [`docs/scale/`](scale/) — metrics specs, dashboard, the
deterministic test framework, and cross-OS/CI material.

---

## Pillar 2 — PROVE SPECTACLE  ·  *tier: NOW*

**Thesis:** the engine doesn't just scale — it looks alive. GPU shaders and
organic motion that double as game mechanics: the **Golden Zone** applied to
rendering, where an optimization *is* the visual feature.

### Frontend foundations (validated)

- **PixiJS (WebGL)** renderer driving large instanced entity counts.
- **Electron desktop** shell, with a **web-distribution path** open.
- **Zero-copy NAPI Float32Array IPC** feeding render state from Rust to the GPU
  pipeline without a serialization tax (see Pillar links / the Rust×JS thesis).

### NOW deliverables

- **Shader-driven organic motion** — procedural, GPU-side animation so 10K+
  creatures move organically with negligible CPU cost. Backend ships positions;
  shaders synthesize the life. See
  [`docs/visuals/ideas/procedural-gait-synthesis.md`](visuals/ideas/procedural-gait-synthesis.md)
  (biologically-grounded gait with allometric scaling).
- **Visual-systems-as-mechanics (Golden Zone)** — rendering choices that are
  also gameplay/biology. The aim: every visual optimization earns a second
  payoff as an observable, emergent behavior, never arbitrary frame-skipping.

**Pillar 2 home:** [`docs/visuals/`](visuals/) — shader designs and visual-system
ideas.

---

## Pillar 3 — PLAY  ·  *tier: NEXT (TBD)*

**Thesis:** emergent gameplay layered on the proven engine.

This pillar is deliberately **TBD**. The engine and spectacle come first; play
is layered on top once scale and visuals are proven. We are not committing to a
gameplay scope or schedule here.

What *is* concrete: a deep, already-written **feature backlog** to draw from.
Pillar 3 fodder lives in:

- [`docs/biology/`](biology/) — DNA-driven traits and behaviors
  (`ideas/`, `todo/`, and shipped `done/` features such as perception, seeking,
  flocking, avoidance). The Golden-Zone opportunities in `biology/todo/` (motion
  detection, hunger gating) sit at the seam of Pillars 2 and 3.
- [`docs/gameplay/`](gameplay/) — gameplay-layer ideas (taming, territory,
  trophy hunting, minimaps, interactions).

Taming and the Drongo species are tracked here as **ideas**, not commitments.

---

## Pillar 4 — PAYOFF  ·  *tier: NEXT (TBD)*

**Thesis:** what this work returns.

- **Now:** a **portfolio showcase** — career signal for engineers and hiring
  managers, demonstrating Rust × JS systems craft (fearless parallelism,
  zero-copy IPC, determinism at scale, observable performance). And **R&D /
  learning** value for the author.
- **Later (open, not committed):** commercial paths remain open. We are not
  picking one here, and nothing in this pillar is scheduled.

Kept honest and brief on purpose: the payoff is the showcase and the learning
*today*; everything beyond that is optionality, not a plan.

---

## Dreamland

> **Explicitly aspirational. A north-star, NOT a schedule.** Nothing here is
> deleted or disowned — it's the long-horizon vision that gives the engine a
> reason to exist beyond the benchmark. None of it is committed work.

The dream is a full game on top of the proven engine:

- **Steam Early Access** distribution.
- **The daughter-rescue campaign** — crash-landed on an alien world; reach your
  daughter's pod across a living, emergent ecosystem. Systemic challenge, not
  scripted spectacle. (Spoiler honored in the design: the daughter is never
  lost.)
- **Taming** — beacon zones → harpoon capture → DNA cloning; build a living
  army/scout network from the simulation's own creatures.
- **Drongos** — an intelligent helper species with social-learning and
  topological perception.
- **Phase 2 MMO** — a persistent, multiplayer cloud world.

Full Dreamland material — strategy, narrative, project spec, MMO — lives in
[`docs/dreamland/`](dreamland/) and is labeled aspirational throughout.

---

## How the pillars relate

```
            PROVE SCALE (NOW) ─────┐
                                   ├──> a proven engine
            PROVE SPECTACLE (NOW) ─┘
                                   │
                          PLAY (NEXT, TBD) ── draws from docs/biology + docs/gameplay
                                   │
                        PAYOFF (NEXT, TBD) ── showcase + R&D now; commercial paths open
                                   │
                        DREAMLAND (DREAM) ── Steam, rescue, taming, Drongos, MMO
```

**Build order:** prove the engine (Pillars 1 + 2) → layer play on it (Pillar 3)
→ realize payoff (Pillar 4) → chase the dream (Dreamland). Each step stands on
the validated one below it.
