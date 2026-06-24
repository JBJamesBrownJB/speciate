# 🚧 Visuals — In Progress (NOW · Pillar 2)

> **Category: 🚧 IN PROGRESS (NOW).** This is an active **NOW-tier** pillar —
> work being built right now, not an idea backlog or a finished log. It is the
> home of **Pillar 2 — Prove Spectacle**. Note: the flagship gait deliverable is
> honestly labeled *"Designed, not yet built"* below — read the status table
> before assuming anything here ships today.
>
> **Legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND
>
> Cross-links: the authoritative NOW/NEXT/DREAM tiering is in
> [`../ROADMAP.md`](../ROADMAP.md) (see Pillar 2). The category convention is
> defined in [`../documentation-standards.md`](../documentation-standards.md).

---

# Pillar 2 — Prove Spectacle

> Part of the Speciate showcase. See `docs/ROADMAP.md` for the four pillars (Prove Scale, **Prove Spectacle**, Play, Payoff) and the Dreamland north-star.

**Tier: NOW**

## The Thesis

Speciate is *"a million-creature artificial-life engine, where Rust's fearless parallelism meets the web's visual playground."* Pillar 1 proves the engine can carry the population. **Pillar 2 proves the population is worth watching.**

The web frontend (PixiJS / WebGL) exists precisely because the browser ecosystem is the richest visual playground on earth: shaders, filters, and GPU instancing are a few lines away. The seam to Rust is zero-copy NAPI `Float32Array` IPC, so the simulation's throughput reaches the screen without a serialization tax.

## The Golden Zone, Applied to Rendering

The project's core design heuristic is the **Golden Zone**: an optimization that *is* also the feature. Pillar 2 applies it to rendering.

The canonical example: procedural gait synthesis. Instead of baked skeletal animations (CPU/memory cost, doesn't scale), creatures are deformed in a **vertex shader** from a handful of per-instance attributes (size, speed, energy). That is the performance win — near-zero CPU, runs on tens of thousands of creatures at once. But the *same* shader produces a gameplay-relevant signal: gait frequency reveals body size, gait irregularity reveals fatigue. A predator can read prey condition from how it moves. **One system, two payoffs.** That is the spectacle we are proving: visuals that are simultaneously the optimization and the mechanic.

When evaluating any rendering idea here, the test is: *can we skip or batch work in a way that also produces an observable, biologically meaningful visual?*

## What's Validated vs. Aspirational

Honest framing for engineers reading this:

| Capability | Status |
|---|---|
| Zero-copy `Float32Array` IPC (Rust → PixiJS) | **Validated** — replaced the old stdio/MessagePack path |
| GPU interpolation (decouple render rate from sim tick) | **Validated** — shipped in Sprint 14 (double-buffer, interleaved vertex geometry) |
| Procedural vertex-shader gait / organic motion | **Designed, not yet built** — the flagship Pillar 2 deliverable |
| Breathing / micro-movement / multi-style gait blending | **Idea / backlog** |

## Material In This Folder

Each document below is an exploration toward the goal of *visual systems that are also game mechanics*. They are framed here as a progression — from the rendering foundation that already shipped, to the flagship organic-motion design, to UI/onboarding spectacle experiments.

### Foundations (shipped — the substrate everything else builds on)

- **`ideas/shader-smooth-and-wiggle.md`** — *Archived.* The original brief for the GPU rendering pipeline: move position interpolation and vertex deformation off the CPU and onto the GPU vertex shader, decoupling the simulation tick from the render rate. Phase 1 (kinematic smoothing) shipped in Sprint 14; Phase 2 (organic wiggle) was superseded by the gait synthesis design. This is the architectural ancestor of the whole pillar — read it for the interleaved-buffer / `mix(start, end, t)` foundation.
- **`ideas/phase-2a-geometry-spec.md`** — The concrete engineering spec that turned the brief above into shipped code: replacing PixiJS's high-level `ParticleContainer` with custom `Geometry` + instanced vertex buffers (START/END positions per creature). This is the seam where the zero-copy NAPI buffer meets the GPU. It establishes the attribute pipeline (`aSize`, `aStartPos`, `aEndPos`, …) that the gait shader later extends. Note: portions describe the pre-shipped state; treat the code samples as design intent and the live renderer as source of truth.

### Flagship (the Golden-Zone deliverable for this pillar)

- **`ideas/procedural-gait-synthesis.md`** — The centerpiece. Generate biologically-grounded gait in the vertex shader from allometric scaling laws (step frequency ∝ size^-0.33, amplitude ∝ speed^0.67), with head-to-tail phase lag and fatigue-driven jitter. The Golden Zone payoff: predators can estimate prey **size from gait frequency** and **fitness from gait irregularity**, so hunting strategy emerges from rendering for free. Includes expert sign-off (shader feasibility, biological validation) and a phased roadmap. This is what "Prove Spectacle" is aiming at.
- **`ideas/shader-animation.md`** — The broader organic-motion umbrella: breathing (state-driven expansion/contraction), undulation (defers to gait synthesis above), and micro-movement Perlin noise so creatures are never robotically still. Framed as "motion is life" at 200K-creature scale with sub-millisecond GPU cost. Sits one level up from gait — gait is its Phase 2.
- **`ideas/motion-blur-and-soft-sprites.md`** — *💡 Idea / deferred.* Soft-edged creature sprites (radial alpha falloff = free analytic antialiasing) plus a per-creature **velocity streak** (motion blur) stretched along each creature's displacement vector — bounded per-creature so it can't wash out at 1M. Inspired by the author's CV Boids demo (alpha-fade trail + additive glow + soft sprites). Golden Zone: streak length reads as speed. Also records the honest `antialias: false` finding and the `low-power → high-performance` GPU lever. Composes with gait synthesis.

### Spectacle & onboarding experiments (the player's first impression)

- **`ideas/opening-portal.md`** — A short visual-direction note for a "vertical CRT / rift" opening sequence: a glowing dot snaps into a vertical beam, then widens into a square portal, with intense orange glow and convex fishbowl distortion. Spectacle applied to onboarding rather than creatures — the first thing a viewer sees.
- **`crt-load/index.html`** — A runnable prototype of that opening: PixiJS + GSAP driving a `Graphics` mask reveal through a hand-written CRT barrel-distortion fragment shader (no external filter dependency). This is the experimental proof-of-concept for `opening-portal.md` — open it in a browser to see the effect. Demonstrates the same "write the shader yourself" muscle the creature work relies on.

## How This Connects To The Other Pillars

- **Pillar 1 (Prove Scale)** supplies the population and the per-creature state (position, velocity, energy) that these shaders consume via the zero-copy buffer. Spectacle is meaningless without the scale to make it impressive, and worthless if it costs CPU the simulation needs.
- **Pillar 3 (Play)** is where gait-as-signal stops being a visual and becomes a mechanic: perception code reading gait jitter to drive predator targeting. The Golden Zone is the bridge — the visual *is* the gameplay.

## Status Badges (placeholder)

These reflect Pillar 1's scale ladder and are static placeholders; Pillar 1's CI will make them live.

![target](https://img.shields.io/badge/target-1M%20creatures-blue)
![linux](https://img.shields.io/badge/Linux-500K%20achieved-success)
![windows](https://img.shields.io/badge/Windows-900K%20%4020Hz-success)
