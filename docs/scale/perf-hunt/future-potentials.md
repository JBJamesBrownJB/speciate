# 📋 Future perf potentials — proven, held wins

> Tested optimizations the gate **KEEP**'d but that are **not yet merged** (a human call,
> recorded as `merge: held` in [`ledger.jsonl`](ledger.jsonl)). Each is real and measured;
> what's holding it is a tradeoff, not the numbers. Listed best-first.

All deltas are **wall p99 median** at **1M creatures / 5 seeds**, back-to-back A/B through the
median gate (`bench_lab::verdict::classify`). Negative = faster. All three are **bit-identical**
(no biology change, no trophic canary).

---

## ✅ MERGED — `native-par-iter-kill-1m-collect` (−6.9 ms confirmed) · `eba30db`

**The biggest single win the hunt has produced.** KEEP'd and measured 2026-06-26; merged same
day after all gate tests passed. Measured post-merge: wall p99 **39.3 ms** at 1M realistic-DNA
(3 seeds), down from **46.2 ms** — a **−6.9 ms** real-world gain (vs −7.26 ms lab measurement;
difference is distribution/seed variance).

- Gate tests shipped: `ComputeTaskPool` thread-count guard + NaN/Inf position sanity check.
- **Re-baseline the perf-hunt** — perception & grid_rebuild (~8.4 ms) are the new top targets.
  `fuse-steering-integrate-system` is now obsolete (same barrier, weaker result).

---

## ✅ MERGED — `perception-compact-active-set` (−2.57 ms) · `c9fe2a2`

Hoist the throttle + `is_active` gate into the serial collect so the parallel dispatch shrinks
~`perception_divisor`×. The biggest single perception win of the 2026-06-25 hunt. **Merged
2026-06-25** after a human-validated 10-min soak (1M/20Hz held ~46→48 ms total tick, under budget;
live perception p99 dropped ~10.6 → 7.45 ms — see `docs/performance/snapshots/win_pop1M_47.5ms_2026-06-25_2210.json`).
**⚠️ Re-baseline the perf-hunt** before the next run — perception detect bars have shifted.

---

## 1. `fuse-steering-integrate-system` — −2.70 ms *(held: profiling + coupling cost)*

Fuse `update_steering_system` + `integrate_motion_system` into one query+closure, deleting a serial
1M-element collect, a fork-join barrier, and a redundant shared-column reload.

- **Result:** wall p99 −2.70 ms (replicated −3.75 ms); takes 1M from ~47.75 → 45.06 ms. `cells_queried` identical.
- **Why held:** (1) **profiling visibility** — steering and movement collapse into one span, blinding
  future per-phase detection on either; (2) **modularity** — couples two previously-decoupled concerns
  into one fat system (schedule simpler, code less modular). See the ledger entry for the full writeup.
- **Interaction:** its fused span overlaps steering targeting — **re-baseline** if merged, and it
  changes the landscape for any future steering hunt.
- **Diff:** not preserved in-repo; recover from the run journal (`wf_420034eb-*`, idea id
  `fuse-steering-integrate-system`) or re-implement from the ledger description.

## 2. `perception-l1-cone-parallel-split` — −1.49 ms solo / **+−0.43 ms stacked** *(held: weak marginal value)*

Hoist the heavy L1 strategic-cone scan into its own `min_len(1)` work-stealing Rayon pass over only
`CanStrategicVision`-marked giants, so a few giants stop straggling behind 63 cheap neighbors.

- **Result (solo):** wall p99 −1.49 ms, perception −1.16 ms.
- **Result (stacked on compact):** marginal contribution drops to **−0.43 ms** — the two share the
  perception dispatch path (the stack is only 74% additive; compact already captures 86% of it).
- **Why held:** the marginal win is small for real added complexity — a new `CanStrategicVision`
  capability marker, a split query, a second fork-join barrier, and a **`perception.range` is static
  post-spawn** invariant that must be maintained (revisit if range ever becomes runtime-mutable).
- **Diff:** preserved at [`candidates/perception-l1-cone-split.diff`](candidates/perception-l1-cone-split.diff).
  Apply with `git apply --recount` (it edits `update_perception_system`, so it conflicts with the
  compact change and needs a hand-merge on top — see the stack-A/B ledger entry for how).

---

## Stack note (additivity)

Measured same-session: **cone-split + compact stack to −2.99 ms wall median = 74% of the −4.06 ms
naive sum** (perception phase 86%). It is *not* one-or-the-other (the stack beats the best single by
0.43 ms), but it is heavily sub-additive. **Compact is the keystone; cone-split is a marginal add-on.**
Full breakdown: ledger id `stack-ab-perception-cone-plus-compact`.

## Test debt spotted

`test_fov_variants_medium_90_crowd` is a **pre-existing flaky test** (~1/6 failures on `main` too —
it asserts a tight 2–5 neighbor count after a chaotic 10-tick feedback loop). Worth tightening or
seeding deterministically; unrelated to the wins above.
