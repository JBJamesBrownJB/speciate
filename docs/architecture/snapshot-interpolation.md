# Snapshot Interpolation — Smooth Motion Across the NAPI Seam 📖

> **Category: 📖 REFERENCE.** Standing explanation of how Speciate renders smooth
> motion from a 20 Hz simulation. For the full problem narrative and the dev-ui
> before/after instrument, see [`../render-pipeline/README.md`](../render-pipeline/README.md);
> for the shipped task records, [`../render-pipeline/done/`](../render-pipeline/done/).

## What

The simulation commits positions at **20 Hz** (one snapshot every 50 ms); the screen
redraws at 60–120 Hz. The renderer's job is to **slide** creatures between successive
sim positions so 20 discrete jumps a second read as one continuous motion. Speciate
does this with **snapshot interpolation**: it buffers snapshots, renders **one tick in
the past**, and drives the slide from a real-time playout clock — *never* resetting the
slide when a new snapshot arrives.

## Why — the NAPI seam is a tiny network

The Rust core and the JS renderer are decoupled producers and consumers joined by the
zero-copy NAPI `Float32Array` double buffer (see [`electron-architecture.md`](./electron-architecture.md)).
Producer and consumer run on independent clocks, so **delivery is jittery** — exactly
the condition networked games face between server and client. The same fix applies:
treat the seam as a tiny network and **render in the past**.

The naive approach — show the latest snapshot and restart the slide on each arrival —
turns delivery jitter into visible stutter:

- a gap **shorter** than the assumed 50 ms → the slide is yanked forward mid-move → **snap**;
- a gap **longer** than 50 ms → the slide finishes and the creature sits frozen → **freeze**.

That snap/freeze alternation *was* the high-population jitter bug
([`../testing/bugs/jitter-high-populations.md`](../testing/bugs/jitter-high-populations.md), resolved).

## How it works (the three ideas)

1. **Render in the past.** Buffer snapshots; don't start playback until one is queued
   *beyond* the pair being shown. There is always a target ahead to roll into, so the
   slide never has to reach 1.0 and wait — which is what removes the freeze.
2. **A continuous playout clock, never reset on arrival.** The slide progress α advances
   with real elapsed time, `α = clock / tickInterval`, and **rolls over** between
   snapshots carrying the remainder (no snap to 0). A new snapshot only *appends* to the
   buffer; it never touches α. On buffer underrun α holds at 1.0 (no extrapolation/overshoot).
3. **Match by creature id.** Creatures spawn and die between snapshots, so `from`→`to` is
   matched by id; a newly-appeared id renders `start = end` (no ghosting), a departed id
   is dropped.

The cost is ~50 ms of added latency (one tick in the past) — imperceptible for a creature
sim. Valve renders networked play 100 ms in the past for the same reason.

This is the second half of a two-part fix. The first half, **push-on-swap**, removed the
*delivery* jitter at the source by replacing a free-running poll with a per-swap event
(doorbell) — see [`../render-pipeline/done/push-on-swap.md`](../render-pipeline/done/push-on-swap.md).
Interpolation makes the renderer robust to whatever jitter the async boundary still leaves.

## Keeping the win — the GC ring

Copying each snapshot into fresh objects every tick would churn ~10M short-lived objects/sec
at 500k creatures — a GC sawtooth that re-introduces the very stutter just removed. So each
snapshot is copied into a pre-allocated **Structure-of-Arrays** slot from a round-robin pool
(`CreatureFramePool`); a steady-state tick allocates nothing. The playout buffer is capped so
slots recycle safely (pool size ≥ cap + 2), guarded by a load-bearing aliasing test. This is
the same pre-allocate-and-reuse discipline the engine uses everywhere on the hot path.

## Implementation

| Concern | Where |
|---------|-------|
| Playout clock (pure, generic, unit-tested) | `apps/portal/src/rendering/SnapshotInterpolator.ts` |
| Renderer wiring + GPU upload | `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts` |
| SoA frame-slot pool (GC ring) | `apps/portal/src/rendering/CreatureFramePool.ts` |
| id-matched start/end buffer builder | `apps/portal/src/rendering/interleavedBuffer.ts` |
| DEV-only verification probe (dev-ui panel) | `apps/portal/src/rendering/InterpolationDiagnostics.ts` |
| Sim-side doorbell (push-on-swap) | `apps/simulation/src/napi_addon/simulation_engine.rs` |

Verification metric: **Stall frames** in the dev-ui Render Pipeline panel (frames frozen
at α = 1.0) — the fix drives it to ~0%. See [`../scale/dev-ui-metrics-reference.md`](../scale/dev-ui-metrics-reference.md).

## Credits

- **Yahn W. Bernier (Valve), GDC 2001** — *"Latency Compensating Methods in Client/Server
  In-game Protocol Design and Optimization"* — introduced **entity interpolation** ("render
  in the past"), shipped in the Source engine as `cl_interp` (default 100 ms behind).
- **Glenn Fiedler** — *"Fix Your Timestep!"* and *"Snapshot Interpolation"* (gafferongames.com)
  — the underlying fixed-timestep + interpolation math (`α = elapsed / dt`, buffered snapshots).

---

**Document Owner:** render pipeline · **Last Updated:** 2026-06-20
