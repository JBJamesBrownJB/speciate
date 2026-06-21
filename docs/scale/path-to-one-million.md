# Path to One Million 🚧

> **Category: 🚧 In progress (NOW) — Pillar 1.** The honest, measured state of the
> million-creature quest. Every number here now traces to the deterministic
> **latency tuning lab** (`apps/simulation/src/bench_lab/`, doc:
> [`latency-tuning-lab.md`](./latency-tuning-lab.md)), which reproduces the
> production engine within ~1% (verified below). See [`README.md`](./README.md) for the ladder.

## Where we are (2026-06-21) — harness-verified

The latency lab now reproduces the engine faithfully. At **900,000 creatures** (random
DNA, spread across the full ±5000 world, perception + behavior throttled to **divisor 8** —
the standard benchmark config) the lab measures **48.3 ms mean** wall-clock tick versus the
engine's own snapshot at **48.6 ms**
([`../performance/snapshots/win_pop900k_48.6ms_randomDNA_2026-06-21_1450.json`](../performance/snapshots/win_pop900k_48.6ms_randomDNA_2026-06-21_1450.json))
— under 1% apart, with steering matched to <1%. The model and reality finally agree, so the
numbers below are reproducible from a seed, not a single manual run.

### Growth curve (seed 1, random DNA, full ±5000 world, divisor 8)

| Population | Mean tick | p99 tick | Within 50 ms? |
|-----------|-----------|----------|---------------|
| 500,000 | 26.2 ms | 28.5 ms | ✅ |
| 700,000 | 36.5 ms | 38.8 ms | ✅ |
| 800,000 | 42.6 ms | 44.2 ms | ✅ (last clean p99) |
| 900,000 | 48.3 ms | 56.9 ms | mean ✅ / **p99 ✗** |
| 1,000,000 | 56.3 ms | 68.2 ms | ❌ |

**Honest ceiling: ~920K by mean, ~830K by p99** — the tail busts 50 ms before the mean does,
so 900K is "within budget but with no headroom" (occasional ticks spill). **1M is ~56 ms mean /
68 ms p99 — about 12% over the budget on the mean.**

Growth is **~O(n¹·¹⁵)** — mildly super-linear. In a *fixed* ±5000 world, density rises with
population (more neighbours per perception, more occupied cells per grid rebuild + L1
aggregation), so per-creature cost creeps up. A constant-density ramp (world growing with
population) would be flatter and reach higher — a benchmark variant worth adding.

## The dominant lever: the cognitive-system throttle

Perception + behavior run on a **frequency throttle** (`FreqConfig`, entity-id bitwise
bucketing): at divisor 8, only 1/8 of creatures perceive each tick. **This is the single
biggest performance lever.** Perception cost is ~linear in creatures-processed-per-tick, and
perception *range* scales super-linearly with body size
(`apps/simulation/src/simulation/perception/components.rs:98`), so a random-DNA world full of
large creatures is dominated by perception. The engine default is now **8** (was 2); running
at 2 quadruples perception load and was the cause of an earlier mis-measured ~590K ceiling.
The throttle is a Golden-Zone optimisation — slower perception means slower reactions, a real
biological cost paid for the speed.

## Where the ~48 ms goes (900K, divisor 8)

| System | Time |
|--------|------|
| Perception | ~14.5 ms |
| Steering | ~13.1 ms |
| Movement | ~8.3 ms |
| Spatial grid rebuild | ~5.9 ms |
| L1 aggregation | ~3.6 ms |
| Behavior transition | ~3.8 ms |

**Per-phase timings sum to ~total wall (gap <0.5%)** — there is **no significant serial-glue
idle between phases.** The earlier "~61% CPU = cores idling in serial stretches" reading is
**refuted**: that 61% was a coarse system-wide `sysinfo` average, not in-tick core occupancy.
Any remaining under-utilisation lives *within* each parallel phase (load imbalance), not
between them. Perception + steering are ~57% of the tick — that's where the last ~12% to 1M is
won or lost.

## Concrete next steps (ranked)

1. **Attack perception (~14.5 ms).** Fattest phase and a Golden-Zone target: size-based skip
   (giants ignore tiny entities), satiated-predator skip, throttle-hoist (don't dispatch Rayon
   tasks for throttled-out creatures), tighter Rayon chunking. A/B each in the lab and watch
   the **p99 tail** — that's what fails first.
2. **Attack steering (~13.1 ms).** Nearly as fat as perception. Reduce per-creature force
   sources; check parallel chunk sizing.
3. **Parallelise the serial within-phase work** — L1 aggregation and the grid rebuild's
   prefix-sum have single-threaded stretches inside their phases.
4. **CI the harness.** The lab is deterministic and seed-stamped — wire `--sweep` /
   `--find-max` into CI so the curve and ceiling regenerate on every commit (the Pillar 1
   "live badge" deliverable).

## Prerequisite for *sustained* 1M (not just a cold-start peak)

A long-running 1M world with creature death + respawn crosses the **f32 id-precision ceiling**
(~16.7M cumulative spawns) and corrupts interpolation matching. Fix before claiming *sustained*
1M, not just a cold-start peak:
[`../testing/bugs/f32-id-precision-ceiling.md`](../testing/bugs/f32-id-precision-ceiling.md).

---

**Document Owner:** Pillar 1 (Prove Scale) · **Last Updated:** 2026-06-21 (harness-verified)
