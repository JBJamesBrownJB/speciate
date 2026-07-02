# Latency Tuning Lab 🚧

> **Category: 🚧 In progress (NOW) — Pillar 1.** A deterministic harness for ruling
> speed/population optimizations in or out empirically. Code: `apps/simulation/src/bench_lab/`.

## Verified faithful (2026-06-21)

The lab reproduces the production engine. At **900K** (the standard config below) it measures
**48.3 ms mean** wall-clock tick vs the engine's own snapshot at **48.6 ms**
([`win_pop900k_48.6ms_randomDNA`](../performance/snapshots/win_pop900k_48.6ms_randomDNA_2026-06-21_1450.json))
— **<1% apart**, with steering matched to <1% and every phase within a few percent. It is a
faithful model, not an argument with reality.

> Getting there took finding the real bug: the lab was running the cognitive-system throttle at
> the engine *old* default (divisor 2) while the benchmark runs **divisor 8**, a 4× perception
> load. That mis-config produced a bogus ~590K ceiling. Lesson: **the throttle divisor is the
> dominant perf lever** — always match it.

## Standard config (what the lab defaults to)

Random DNA, spread across the **full ±5000 world** (`MAX_WORLD_SIZE`), perception + behavior
throttled to **divisor 8** (`FreqConfig`, now the engine default). This is the real benchmark
methodology. The binary defaults match it: `--half-x 5000 --half-y 5000`, random DNA, divisor 8
applied by `build_world`.

## What it does

Three measurements, three jobs:

1. **Headline KPI — max sustainable population.** `--find-max` does a coarse-bracket →
   bisection search for the largest population whose **p99 tick ≤ 50,000 µs**
   (`TICK_BUDGET_US`). This is the undeniable scoreboard number.
2. **Growth curve.** `--sweep --sweep-from --sweep-to --sweep-step` measures p99 at evenly
   spaced populations — the *shape* of tick-time vs count (currently ~O(n¹·¹⁵), super-linear
   from rising fixed-world density). Use this, not the find-max trail, to see the growth shape.
3. **Diagnostic — per-phase A/B.** A fixed-population run captures per-phase timings
   (perception/steering/movement/grid/L1/behavior) plus wall-clock total, so a change can be
   attributed to a phase, not guessed at.

## What metric the budget keys on

The lab drives `Simulation::update()` directly (headless, no NAPI engine loop). In that
path the authoritative tick latency is **`wall_total`** — the real wall-clock of each
`sim.update`, populated in every build. The instrumentation field `total_tick_us` and
`export_positions_us` are timed inside the NAPI engine loop and read **0** in the lab, so
they never drive pass/fail or the max-pop search — they remain in the report for
attribution only. Consequence: the lab's wall-clock is the core-schedule tick time; the
production engine additionally spends ~2 ms/tick on export+IPC, so the lab number is
slightly optimistic versus the full engine. Treat the lab as the core-tick microscope.

## Why it is trustworthy

- **Deterministic worlds.** `(population, seed, distribution, extents)` reproduce the
  exact same initial world (`Dna::random_seeded` + `StdRng`). A/B comparisons change the
  code, not the dice. Pin the seed. (The hot loop is Rayon-parallel, so tick *evolution*
  may drift; the seeded *spawn* is exact, which is what benchmarking needs.)
- **Tail, not mean.** Pass/fail is p99 (`BudgetMetric::P99`) — the mean hides the
  dropped beats that live in the tail.
- **Per-phase attribution.** Built with `--features dev-tools`, the lab reads
  `Simulation::get_system_timings()`; the chained `.after()` schedule means a win only
  shows in wall time if it was on the critical path, so always read the per-phase diff.

## Workload note (read before trusting a max-pop number)

`Uniform` spread is the *cheap* density regime. Emergent flocking clusters the world,
raising perception cost at equilibrium, so a max-pop measured on a fresh uniform spread
can be optimistic. Run `--clustered` as the adversarial case for any headline claim.

## Commands

```bash
cd apps/simulation

# Fixed-pop diagnostic (per-phase attribution; A/B a change by diffing two --out reports)
cargo run --release --features dev-tools --example latency_lab -- \
  --pop 200000 --seed 1 --samples 60 --warmup 20 --out /tmp/before.json

# Growth curve (the shape of tick-time vs population)
cargo run --release --features dev-tools --example latency_lab -- \
  --sweep --sweep-from 100000 --sweep-to 1200000 --sweep-step 100000 --seed 1 --out /tmp/curve.json

# Headline: find the max sustainable population
cargo run --release --features dev-tools --example latency_lab -- \
  --find-max --low 700000 --high 1100000 --coarse-step 100000 --tolerance 25000

# Adversarial clustered workload
cargo run --release --features dev-tools --example latency_lab -- \
  --pop 200000 --clustered --clusters 32 --spread 150
```

Latest verified curve (seed 1, random DNA, full world, divisor 8): 500K = 26 ms, 800K = 43 ms,
900K = 48 ms mean (p99 57 ms), 1M = 56 ms mean (p99 68 ms). **Ceiling ~920K by mean / ~830K by
p99; 1M ≈ 12% over budget.** See [`path-to-one-million.md`](./path-to-one-million.md).

## The honest gaps

- Hardware PMU counters (IPC, cache misses) remain Linux-only (`perf-event`); the lab's
  per-phase µs are software timers, valid cross-platform but blind to *why* a phase is slow.
- The lab measures the engine in-process, without the Electron/render pipeline. It is the
  tick-budget microscope, not an end-to-end frame-delivery test.
- Not yet a CI regression gate: that needs committed baselines + relative (ratio) thresholds
  + a quiet runner. The `LabReport` JSON schema and `diff_reports` are built to enable it.

**Document Owner:** Pillar 1 (Prove Scale)
