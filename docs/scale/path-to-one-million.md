# Path to One Million 🚧

> **Category: 🚧 In progress (NOW) — Pillar 1.** The honest, measured state of the
> million-creature quest. Every number here now traces to the deterministic
> **latency tuning lab** (`apps/simulation/src/bench_lab/`, doc:
> [`latency-tuning-lab.md`](./latency-tuning-lab.md)), which reproduces the
> production engine within ~1% (verified below). See [`README.md`](./README.md) for the ladder.

## Where we are (2026-06-21) — 🎉 ONE MILLION achieved

**1,000,000 creatures sustained at 20 Hz on Windows.** Live engine snapshot:
**48.5 ms mean tick · 49.4 ms p99 · 49.4 ms max** — the *entire* distribution under the 50 ms
budget — with **0 render stall frames** (σ 1.15 ms) and ~2.8 GB memory. Config: **realistic
(log-normal) DNA, perception + behavior throttled to divisor 8, full ±5000 world.** Evidence:
[`../performance/snapshots/win_pop1M_48.5ms_2026-06-21_2333.json`](../performance/snapshots/win_pop1M_48.5ms_2026-06-21_2333.json).

**Cross-checked, not anecdotal.** The deterministic latency lab independently reproduces the
engine within ~5% per-phase at 1M (perception 9.15 ↔ 9.77 ms, steering 13.55 ↔ 13.4 ms),
confirming the number is real and seed-reproducible — not a single lucky frame.

**Honest framing — it's at the wall, no headroom.** The live run was a clean pass, but the
lab's multi-seed p99 (mean-of-p99s **51.4 ms**, worst seed 52.2 ms, noise floor 0.6 ms) shows
the tail hovers right at 50 ms seed-to-seed. This is a *"just made it"* million, not a
comfortable one.

**What crossed the line: biology, not engine-polish.** Switching "random DNA" from a UNIFORM
size smear (~half the population 5 m+ giants, each doing the expensive O(range²) long-range
perception scan) to a **realistic log-normal pyramid** (most creatures small, ~1–2% giants)
cut perception from ~14.5 ms to **9.15 ms** — the slice that crossed 50 ms. `cellsQueried`
dropped ~47% (the causal proof). The incremental-grid (shelved for a resync stutter) and the
perception micro-ops (below the noise floor) didn't get there; a single biology decision did.
See [`../biology/biology-notes.md`](../biology/biology-notes.md).

**The next lever is now steering (~13.5 ms)** — it overtook perception as the fattest phase, so
that's where headroom *above* 1M lives.

> **Sustained vs cold-start.** This is a cold-start million that held 20 Hz across the captured
> window. An *indefinitely*-running million with creature death + respawn still crosses the
> **f32 id-precision ceiling** (~16.7M cumulative spawns) and corrupts interpolation — the one
> gate between "we hit a million" and "we run a million forever":
> [`../testing/bugs/f32-id-precision-ceiling.md`](../testing/bugs/f32-id-precision-ceiling.md).

### Growth curve — UNIFORM DNA (the heavier pre-realistic workload, for reference; seed 1, divisor 8)

| Population | Mean tick | p99 tick | Within 50 ms? |
|-----------|-----------|----------|---------------|
| 500,000 | 26.2 ms | 28.5 ms | ✅ |
| 700,000 | 36.5 ms | 38.8 ms | ✅ |
| 800,000 | 42.6 ms | 44.2 ms | ✅ (last clean p99) |
| 900,000 | 48.3 ms | 56.9 ms | mean ✅ / **p99 ✗** |
| 1,000,000 | 56.3 ms | 68.2 ms | ❌ |

**On the heavy uniform workload the ceiling was ~920K by mean / ~830K by p99** (1M ≈ 56 ms mean
/ 68 ms p99, ~12% over). That is precisely the gap realistic DNA closed: the same 1M that sat
12% over on uniform DNA lands at 48.5 ms on the realistic log-normal distribution. The uniform
curve remains the conservative "all-giants" stress baseline; realistic DNA is the believable
(and now budget-fitting) standard.

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

> The full try → test → keep/ditch experiment tracker (15+ levers across mechanical, biological
> Golden-Zone, and structural, with the measurement protocol) lives in
> [`optimization-checklist.md`](./optimization-checklist.md). The headline lever: trimming
> large-creature perception range (lower the allometry exponent) — the fattest phase *and* a
> biological-correctness fix.

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
