# Snapshot Interpolation (render in the past)

**Status:** ✅ Implemented 2026-06-20 — the second of the two-part fix; built on [`push-on-swap.md`](./push-on-swap.md). Motion is now visibly smooth ("butter"). The algorithm is promoted to reference: [`../../architecture/snapshot-interpolation.md`](../../architecture/snapshot-interpolation.md).
**Dependencies:** [`push-on-swap.md`](./push-on-swap.md) (steady ~50 ms delivery).
**Area:** Render (TypeScript / portal).

**Implementation:** `apps/portal/src/rendering/SnapshotInterpolator.ts` (the pure render-in-the-past playout clock, generic over the snapshot payload) wired into `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts`. Tests: `SnapshotInterpolator.test.ts`, plus the 26 renderer tests held unedited. A follow-up GC optimization — pooling each snapshot into pre-allocated Structure-of-Arrays slots — landed alongside it: `apps/portal/src/rendering/CreatureFramePool.ts` + `interleavedBuffer.ts` (see §"GC ring" below).

## Goal

Make the renderer **robust to delivery jitter** by adopting Valve-style **entity interpolation**: keep a small buffer of timestamped snapshots, render ~1 tick **in the past**, and drive the interpolation α from a **real-time clock between two snapshots** — *never* resetting α when a snapshot arrives. Background + origin: [`../README.md`](../README.md) §5.

**Credit:** entity interpolation / "render in the past" is from **Yahn W. Bernier** (Valve Software), *"Latency Compensating Methods in Client/Server In-game Protocol Design and Optimization,"* GDC 2001 — shipped in the Source engine as `cl_interp`. Underlying timestep math: **Glenn Fiedler**, *"Fix Your Timestep!"* / *"Snapshot Interpolation."* Full citations in [`../README.md`](../README.md) §7.

## Why (the problem it removed)

The old renderer treated "a snapshot arrived" as its clock: it reset α to 0 on each arrival and advanced α by `deltaMS / fixed-50ms`. Any wobble in arrival timing became a visible **snap** (α reset mid-slide) or **freeze** (α stalls at 1.0). Push-on-swap killed the delivery jitter, but reset-on-arrival meant even a small residual wobble could still freeze a frame — so the renderer itself had to stop using arrival as its clock.

## Design (as built)

The fix is a small, pure playout clock (`SnapshotInterpolator<T>`) — *not* an extension of the old `InterpolationBufferManager` (now dead code). Generic over the snapshot payload `T`, so the timing is unit-testable with no position data.

- **Render in the past.** Snapshots `push()` into a queue; the clock won't emit a segment until **3** are buffered (`START_DEPTH`), so there is always one snapshot *beyond* the pair being shown. Rendering ~1 tick behind is what removes the end-of-tween stall — α never has to reach 1.0 and wait.
- **Continuous clock, never reset on arrival.** `advance(deltaMs)` rolls α forward (`α += deltaMs / tickIntervalMs`); when it crosses 1.0 it **rolls over** to the next pair carrying the remainder, rather than snapping to 0. A new snapshot only *appends* — it never touches α. This is the core invariant.
- **Interpolate the oldest buffered pair.** `current()` yields `{from, to, alpha}` = the oldest two queued snapshots + the clock. The existing GPU shader lerp is reused unchanged — only how α and the start/end buffers are chosen changed.
- **Underrun = hold.** If the clock outruns the buffer, α clamps at 1.0 (hold at the newest); no extrapolation, no overshoot pops.
- **Match creatures by id.** Creatures spawn/die between snapshots, so `from`→`to` is matched by creature **id**; a newly-appeared id gets `start = end` (no ghosting), a departed id is dropped.

### GC ring (follow-up optimization)

Copying each snapshot into a fresh object array every tick is ~10M short-lived objects/sec at 500k creatures — a GC sawtooth that would re-introduce stutter. So snapshots are pooled: a round-robin ring of pre-allocated **Structure-of-Arrays** slots (`CreatureFramePool`), filled through existing typed arrays with no per-creature allocation. The interpolator gained a `maxQueue` cap so queued slots can be recycled safely; the renderer sizes the pool to `maxQueue + 2`. A load-bearing aliasing test (`framePoolAliasing.test.ts`) guards the sizing rule.

## Metric change: α@reset removed

Once α no longer resets on arrival, the panel's old **α@reset** metric measured nothing — so it was **removed** (probe, dev-ui types, panel, and tests). **Stall frames** is the render-side verification signal that survives: it directly counts frozen frames (α pinned at 1.0), which is exactly what the fix drives to ~0.

## Testing (automated, written first — TDD; pure logic, strong vitest fit)

- **`SnapshotInterpolator.test.ts`** drives the clock with a synthetic stream and asserts: renders one tick in the past; α derived from the clock; α **never reset on arrival**; continuous roll-over carrying the remainder; underrun holds at 1.0 (no overshoot); `maxQueue` caps the buffer (drops oldest, never resets α).
- **`CreatureFramePool.test.ts`** / **`interleavedBuffer.test.ts`** pin the SoA copy, the no-alloc reuse contract, and id-matched interleave.
- **`framePoolAliasing.test.ts`** models the renderer's ref lifecycle and proves the pool sizing is sound.
- The **26 existing renderer tests pass unedited** (the public `CreatureData[]`-in / primitives-out contract is unchanged).

## Results (measured live in the Render Pipeline panel, single creature)

| Metric | Before | After this task |
|--------|--------|-----------------|
| **Stall frames** | ~22% (~15% after push-on-swap alone) | **~0%** ✅ |
| **Visible motion** | snap + freeze | **smooth ("butter")** ✅ |
| **Snapshot gap σ** | wobbles | may still wobble on delivery — but motion is smooth **regardless** (the whole point) |
| **α@reset** | 0.84, dips to 0.60 | **metric removed** (no longer meaningful — see above) |

## Acceptance Criteria — met

- ✅ Tests above pass; renderer suites green; portal suite 432 → 444.
- ✅ With one creature, motion is visibly smooth (no snap/freeze) — user-confirmed.
- ✅ Stalls ~0%; no overshoot artifacts.
- ✅ The ~50 ms added latency (one tick in the past) is accepted — imperceptible for a creature sim.
- ✅ Algorithm promoted to [`../../architecture/snapshot-interpolation.md`](../../architecture/snapshot-interpolation.md) + a root `README.md` mention (credit Valve/Bernier + Fiedler).

---

**Document Owner:** render pipeline · **Last Updated:** 2026-06-20
