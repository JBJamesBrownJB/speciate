# Snapshot Interpolation (render in the past)

**Status:** 📋 Planned — **do first** (the high-impact render fix; stands alone).
**Dependencies:** none required. Timestamps come from snapshot **arrival time** initially; [`push-on-swap.md`](./push-on-swap.md) is a later refinement that swaps in exact sim-tick timestamps and removes duplicates.
**Area:** Render (TypeScript / portal).

## Goal

Make the renderer **robust to delivery jitter** by adopting Valve-style **entity interpolation**: keep a small buffer of timestamped snapshots, render ~1 tick **in the past**, and drive the interpolation α from a **real-time clock between two snapshots** — *never* resetting α when a snapshot arrives. Background + origin: [`../README.md`](../README.md) §5.

**Credit:** entity interpolation / "render in the past" is from **Yahn W. Bernier** (Valve Software), *"Latency Compensating Methods in Client/Server In-game Protocol Design and Optimization,"* GDC 2001 — shipped in the Source engine as `cl_interp`. Underlying timestep math: **Glenn Fiedler**, *"Fix Your Timestep!"* / *"Snapshot Interpolation."* Full citations in [`../README.md`](../README.md) §7.

## Why (the problem it removes)

Today the renderer treats "a snapshot arrived" as its clock: it resets α to 0 on each arrival (`apps/portal/src/rendering/InterpolatedCreatureRenderer.ts:247`) and advances α by `deltaMS / fixed-50ms` (`:263`). Any wobble in arrival timing therefore becomes a visible **snap** (α reset mid-slide) or **freeze** (α stalls at 1.0). Even after push-on-swap, the async seam adds some residual jitter — this fix absorbs it.

## Design (high-level)

- **Buffer snapshots with timestamps.** Extend `InterpolationBufferManager` (`apps/portal/src/rendering/InterpolationBufferManager.ts`) to retain the last few snapshots, each stamped with its **arrival time** (`performance.now()` when a *distinct* snapshot is received) — not just previous+current. (Later, push-on-swap replaces arrival time with the exact sim tick.)
- **Render clock, not arrival clock.** Maintain a clock that advances by real frame `deltaMS`, targeting `now − interpolationDelay` where the delay ≈ **1 tick (50 ms)** (Valve uses 100 ms). Arrival of a snapshot only *appends to the buffer* — it never resets the clock or moves the creature.
- **Interpolate the bracketing pair.** Each frame, pick the two buffered snapshots that straddle the render clock and set α from their timestamps (`α = (clock − A.t) / (B.t − A.t)`). The existing GPU shader lerp (`InterpolatedCreatureRenderer.ts:167`) is reused unchanged — only how α and the start/end buffers are chosen changes.
- **Underrun = hold.** If the clock outruns the newest snapshot (buffer ran dry), hold at the newest position. Do **not** extrapolate (avoids overshoot pops).
- **Metric note:** once α no longer resets on arrival, the panel's `α@reset` loses its old meaning. Re-point it to **interpolation continuity** (e.g. fraction of frames with a valid bracketing pair, or α monotonicity) and update the probe (`apps/portal/src/rendering/InterpolationDiagnostics.ts`) + panel accordingly.

## Testing (automated, written first — TDD; pure logic, strong vitest fit)

- **`vitest` (portal), driving the interpolation with a synthetic stream:** feed timestamped snapshots at **uneven arrival times** (simulate σ≈16 ms) plus a stepped render clock, and assert:
  - α is derived **from the clock** and is **monotonic within each snapshot interval**.
  - α is **never reset to 0 on arrival** (the core invariant).
  - **No backward position jump** between consecutive rendered frames (no snap).
  - On buffer **underrun**, the position **holds** (no overshoot / extrapolation).
  - With a *steady* 50 ms stream, output matches the old smooth behaviour (no regression).
- Update/keep the existing `InterpolatedCreatureRenderer.test.ts` / `InterpolationBufferManager.test.ts` green against the new model.

## Expectations (verify live in the Render Pipeline panel, single creature)

| Metric | Before | After this task |
|--------|--------|-----------------|
| **Lerp completion / continuity (α)** | 0.84, dips to 0.60 | **pinned ~1.0** (α sparkline rides the green target line) |
| **Stall frames** | ~22% | **~0%** |
| **Snapshot gap σ** | wobbles | may still wobble on delivery — but motion is smooth **regardless** (the point) |
| **Visible motion** | snap + freeze | **smooth** |

## Acceptance Criteria

- Tests above pass; existing renderer suites green.
- With one creature, motion is **visibly smooth** (no snap/freeze).
- Panel: α green / pinned, stalls ~0%; no overshoot artifacts.
- The ~50 ms added latency is documented and accepted.
- On success: promote the algorithm from [`../README.md`](../README.md) to `docs/architecture/` + a root `README.md` mention (credit Valve/Bernier + Fiedler).

---

**Document Owner:** render pipeline · **Last Updated:** 2026-06-20
