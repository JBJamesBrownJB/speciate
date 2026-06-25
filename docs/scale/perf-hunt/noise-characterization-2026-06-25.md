# 📖 Noise characterization — why the 1M gate can't see small wins (2026-06-25)

> **TL;DR** The verdict's noise floor was the wrong denominator (across-seed *world*
> variance, irrelevant to an A/B). We fixed it to a paired-difference floor (Common
> Random Numbers). But measuring on real 1M data revealed the floor is **not** dominated
> by world variance — it's dominated by **run-to-run drift** (±2.3 ms wall between two
> identical runs), which seed-pairing cannot cancel. The gate is now *honest*, not
> *tighter*. The real lever for amplifying small signals is a **stabler statistic than
> p99**, not the gate math.

## What changed (Tier 1 — paired seeds)

`bench_lab::evidence_from_reports` previously judged a change against one arm's
*across-seed* p99 std. That conflates two things:

- **World variance** — seed 99's world genuinely costs more than seed 2025's. Identical
  in baseline and candidate, so it tells you nothing about whether the change helped.
- **The change's effect** — the thing we actually want to detect.

Baseline and candidate run the **same seeds**, so the world component is shared. Pairing
the per-seed p99s (`dᵢ = candᵢ − baseᵢ`) cancels it: the signal (`mean(dᵢ)`) is unchanged,
but the noise floor becomes `std(dᵢ)` — the reproducibility of the *difference*. Per-seed
p99 vectors are now serialized so the cross-process `--verdict` can pair (commit `505e591`).

On synthetic data where world variance dominates, this is a 40× noise reduction
(3266 µs → 82 µs for the same signal). **On real 1M data it is not** — see below.

## The measurement: same binary, back-to-back A/B, 1M, the hunt's seeds

A null change (identical binary as both arms) measures pure noise. If the lab were
clean, every `B−A` would be ≈0.

```
seed     wallA    wallB    B-A     | steerA  steerB   B-A
11       51574    48072   -3502    | 14510   13938   -572
42       50842    48519   -2323    | 14366   14584   +218
99       52575    49619   -2956    | 15709   14267   -1442
137      50680    53608   +2928    | 15035   16376   +1341
2025     48757    48397    -361    | 14151   14132    -19

WALL  across-seed std A=1257  B=2050   | paired-diff std = 2340
STEER across-seed std A=559   B=884    | paired-diff std =  917
```

Two **identical** runs disagree by **−3.5 ms to +2.9 ms per seed**. The world ranking
isn't even preserved between runs (A's costliest seed is 99; B's is 137). So at 1M:

```
run-to-run jitter (±2.3 ms wall)  >  world-to-world variance (±1.3 ms)
```

Pairing removes the smaller term (world variance) and the dominant term (drift between
two separately-timed runs) survives — which is why the paired floor (2340 µs) is *larger*
than the old single-run across-seed floor (1257 µs). Pairing didn't add noise; it
**revealed** the true reproducibility floor the old metric was hiding.

## Implications

1. **The old across-seed floor was over-optimistic.** It ignored run-to-run drift
   entirely, so it under-reported the noise and would pass irreproducible wins.
2. **Sub-2.3 ms wall "wins" at 1M are not currently trustworthy.** Concretely: the
   2026-06-25 hunt's `size-gated-steering-reaction-latency` candidate (wall p99
   −2.507 ms, flagged as the one real win) is **within** the ±2.3 ms run-to-run floor.
   It should be treated as *unconfirmed*, not a banked win, until the floor drops.
3. **Tier 1 is worth keeping anyway** — it's the statistically correct denominator and
   it's honest in both directions (it tightens the floor when world variance dominates,
   e.g. smaller pops, and exposes drift when that dominates). It just isn't the lever
   that amplifies signals in *this* regime.

## The real lever (next): a stabler statistic than p99

p99 of 30–60 samples is essentially the top 1–2 observations — its sampling variance
alone is ~±1 ms. That, plus thermal drift between runs, is the noise. Options, in order
of expected payoff:

1. **Detect on a trimmed mean / p95 with many more samples**; keep p99 only for the
   50 ms *budget* check. Detection and budget need not use the same statistic.
2. **In-process sample-level A/B** — baseline vs candidate in one warm process, same
   seed, alternating ticks, so thermal state *and* world cancel per-tick. The only
   thing that truly removes drift, but it needs both code paths in one binary.
3. **Fixed clocks / turbo-off / core-pinning** — shrink drift at the source (and likely
   steadier on the undervolted dev rig). See `windows-parity-strategy.md`.

## What we measured before choosing (samples 60 vs 240, same null A/B)

Per-seed **run-to-run paired-diff std** (the noise that limits detection), µs:

| metric | p99 | p95 | median (p50) | mean |
|---|--:|--:|--:|--:|
| wall    | 2377 | 1786 | **812** | 832 |
| steering |  546 |  377 | **250** | 235 |

- The **median is ~2.3–2.9× quieter than p99** for the same signal; p95 is only a half-measure;
  median ≈ mean. → detect on the median.
- **More samples backfired:** at samples=240 every statistic's drift ballooned to ~2.5 ms — longer
  runs heat the chip more. → don't sample harder on this rig.

## ✅ Implemented (Tier 2) — median detection + p99 guard

`classify` / `evidence_from_reports` now source detection (phase) and banking (wall) deltas + noise
floors from the **paired per-seed median**; the per-phase **p99** means still drive the strict
regression guard (commit `8255f67`). End-to-end on a real 1M null A/B, the gate's reported wall noise
floor fell **2340 µs (p99 paired) → 124 µs (median paired)** — and the null change still correctly
Ditches (no false positive). Sub-~2 ms median wins that p99 buried are now detectable.

**Still open (the residual):** ~120–810 µs of run-to-run drift remains (varies with machine heat) and
only Tier 4/5 (fixed clocks / in-process sample-level A/B) can take it. That is the next lever.
