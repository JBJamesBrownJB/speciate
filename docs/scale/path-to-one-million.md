# Path to One Million 🚧

> **Category: 🚧 In progress (NOW) — Pillar 1.** The honest, measured state of the
> million-creature quest and the concrete levers left. Evidence-driven; every number
> here traces to a committed snapshot. See [`README.md`](./README.md) for the ladder.

## Where we are (2026-06-20)

A single Windows run sustained **~900,000 creatures at a steady 20 Hz**. The tick is at
the edge of the budget and the render pipeline is still smooth.

Evidence: [`../performance/snapshots/win_pop900k_49.4ms_2026-06-20_2352.json`](../performance/snapshots/win_pop900k_49.4ms_2026-06-20_2352.json)
(16-core machine, single run, not yet CI-benchmarked).

| Signal | Value | Read |
|--------|-------|------|
| Population | 900,000 (rock-steady) | — |
| Tick rate | 20 Hz held | no dropped beats |
| **Total tick** | **~49.4 ms** of the 50 ms budget | **~0.6 ms headroom — at the wall** |
| Render: stall frames | **0** | smooth |
| Render: snapshot σ | **0.8 ms** | (was ~16 ms pre-fix) — "render in the past" scales |
| Process memory | ~3.5 GB | comfortable; ~3.9 GB projected at 1M |
| CPU utilisation | **~61%** across 16 active cores | **the lever — see below** |

The buffer ceiling is already 1M (raised this session), so the remaining gap is **tick
budget**, not capacity.

## Where the 49.4 ms goes

| System | Time | Share |
|--------|------|-------|
| Perception (neighbour detection) | ~15.0 ms | ~30% |
| Steering (fused wander/seek/avoid forces) | ~12.5 ms | ~25% |
| Movement (physics integration) | ~8.0 ms | ~16% |
| Spatial grid rebuild | ~5.5 ms | ~11% |
| L1 aggregation | ~3.0 ms | ~6% |
| Behavior transition | ~3.0 ms | ~6% |
| Export positions | ~2.4 ms | ~5% |

**Perception + steering are ~55% of the tick** — that's where the budget is won or lost.

## The lever: ~61% CPU at the wall

All 16 cores are engaged, yet average utilisation is only ~61%. A 49 ms wall-clock tick
that only keeps the cores ~61% busy means cores are **idling inside the tick** — serial
stretches or load imbalance between parallel and serial systems. Closing that gap is the
most promising route to 1M at 20 Hz: the work to run 900K is *already cheaper than 49 ms*
if it were spread evenly. **Caveat:** this is an estimate from a coarse utilisation metric;
the real serial bottleneck must be pinned with a profiler before chasing it.

## Concrete next steps (ranked, unstarted)

1. **Profile the tick to find the idle.** Linux `perf`/PMU (the cockpit panel) or a
   Windows profiler (WPA/Tracy) to see *which* phase leaves cores idle — confirm or refute
   the 61% read before optimizing. (Instrumentation home: `docs/scale/`, agent: instrumentation-ian.)
2. **Attack perception (~15 ms).** It's the fattest system and a Golden-Zone target:
   size-based skip (giants ignore tiny entities), satiated-predator skip, tighter FOV/range
   genes, coarser Rayon chunking. Each is a perf win that *is* a biological feature.
3. **Attack steering (~12.5 ms).** Frequency-throttle where physics allows; reduce per-creature
   force sources; check parallel chunk sizing.
4. **CI-validate the number.** Turn the ~900K peak into a reproducible, cross-platform
   CI benchmark (Pillar 1 deliverable) so the badge stops being a placeholder.

## Prerequisite for *sustained* 1M (not just a peak)

A long-running 1M world with creature death + respawn will cross the **f32 id-precision
ceiling** (~16.7M cumulative spawns) and corrupt interpolation matching. Fix that before
claiming sustained 1M, not just a cold-start peak:
[`../testing/bugs/f32-id-precision-ceiling.md`](../testing/bugs/f32-id-precision-ceiling.md).

---

**Document Owner:** Pillar 1 (Prove Scale) · **Last Updated:** 2026-06-20
