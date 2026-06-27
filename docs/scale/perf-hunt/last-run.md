# Perf Hunt Report — 2026-06-27

**Baseline:** 1M pop, 5 seeds (11/42/99/137/2025), realistic-dna, half-world 5000x5000.
Wall p99 mean-of-p99s = **35.529 ms** (std 2.923 ms, worst seed 40.026 ms).
Ideas considered: 12 | Implemented: 12 | Keeps: 1 | Defers: 1 | Ditches: 10 | Bundle: none (mutually exclusive pair).

---

## Headline Table

| Idea | Scope | Phase | Verdict | Dwall p99 (ms) | Dphase (ms) |
|------|-------|-------|---------|---------------|-------------|
| Pre-filter behavior_transition (compact-active-set) | engine | behavior | **KEEP** | **-4.882** | **-2.375** |
| Fuse behavior_transition into steering | architectural | behavior | DEFER | -5.681 | -3.858 |
| GridFixed ZST marker for catatonic grid skip | architectural | grid_rebuild | DITCH | +1.217 | +0.364 |
| Perception radius prefilter (cbrt threshold) | biological | perception | DITCH | +0.096 | +0.096 |
| Motion camouflage / freeze-skip | biological | perception | DITCH | -0.095 | +0.048 |
| Avoidance skip for catatonic/waiting creatures | engine | steering | DITCH | +1.921 | +0.779 |
| FOV squared-distance (kill sqrt in collect_cells) | engine | perception | DITCH | -0.049 | -0.052 |
| Bounded K-slot neighbor buffer (kill partial sort) | engine | perception | DITCH | -0.302 | +0.189 |
| Hunger-gated L1 range halving | biological | perception | DITCH | +1.156 | 0.000 |
| Hoist max_force/mass/max_speed CSE scalars | engine | steering | DITCH | +1.125 | +0.045 |
| Sort L0 cells by L1 parent before aggregation | architectural | l1_aggregation | DITCH | +14.832 | +15.027 |
| fast_inv_sqrt in L1 cone normalization | engine | perception | DITCH | -2.274 | -0.831 |

---

## KEEPS

### behavior-compact-active-set — Pre-filter behavior_transition before par_iter_mut dispatch

**Wall p99 delta: -4.882 ms (replicated -4.951 ms). Phase delta: -2.375 ms. Phase SNR: 30x.**

**What it does.** Before the `par_iter_mut` dispatch in `behavior_transition_system`, it compacts the ECS entity list down to only those in the current throttle bucket — the ~1/behavior_divisor entities due to run this tick. The remaining ~(behavior_divisor-1)/behavior_divisor no-op Rayon work units are never dispatched.

**Why it works beyond the phase.** The `behavior` phase directly measured -2.375 ms. The wall improvement (-4.882 ms) is ~2x larger than the phase saving, because eliminating no-op Rayon tasks frees CPU cache and scheduler headroom that benefits the downstream perception, steering, and movement phases within the same tick.

**Tradeoffs / costs:**

- The serial ECS row scan to build the compact set is still O(1M) — unavoidable until the ECS natively exposes the throttle bucket as a column. This is a roughly constant overhead of a few hundred microseconds regardless of behavior_divisor.
- At `behavior_divisor=1` (every entity runs every tick) the filter is a no-op and returns the full list — zero harm but zero gain. Verify the production benchmark config has `behavior_divisor > 1` before landing.
- **Mutually exclusive with fuse-behavior-transition-into-steering (below).** Both attack the same `behavior_transition_system`. Implement exactly one.

**No biology change.** The entities processed are identical — just filtered to the same subset that would have been processed anyway. Bit-identical output. No trophic canary required.

**Branch:** ready to merge. No hold.

---

## DEFERS (parked — re-measure before deciding)

### fuse-behavior-transition-into-steering — Fold behavior_transition into the steering closure

**Wall p99 delta: -5.681 ms (tool median). Raw 1M signal: ~7.7 ms (36503->28840 us). Phase delta: -3.858 ms.**

**Signal is real, noise is the problem.** The behavior phase collapsed entirely (4874 us -> 0 us) and the wall dropped ~7.7 ms raw. The DEFER verdict is purely a baseline stability issue: the baseline run had a worst seed at 42306 us vs a mean of 36503 us (std 3366 us). The candidate run had std 703 us — the candidate is rock-solid, the baseline was noisy. Wall noise of 3.396 ms exceeds the median improvement by the tool's 2x threshold.

**Triage (500k) returned KEEP** with dWallMedian -2.024 ms / dPhase -1.656 ms at much lower noise. The directional signal is consistent from 500k triage through 1M escalation.

**What it does.** Eliminates one serial 1M-entity `Vec::collect()` and one full Rayon fork-join barrier per tick by running the behavior-transition logic inside the existing steering `par_iter_mut` closure at the top of each per-entity call.

**Tradeoffs / costs:**

- `behavior` and `movement` phase metrics are lost as diagnostic signals — both phases collapse to 0, making future per-phase regression detection blind in those dimensions. You would bank entirely on the wall median.
- `Brain` gains a `mut` borrow inside the steering query — verify no other system holds a conflicting borrow on `Brain` mid-tick in the same schedule.
- If `behavior_divisor` and steering divisor ever need to differ, the fused body must be split back out.
- **Mutually exclusive with behavior-compact-active-set.** The directional wall win here is slightly larger (-5.681 ms vs -4.882 ms) but compact-active-set is clean and measurable today. If the fuse re-measures KEEP on a stable baseline, it supersedes the compact approach.

**Action:** Re-measure after the machine baseline is calm (baseline std < 1 ms). The raw -7.7 ms signal would make this the second-largest single-run win in the ledger after native-par-iter (#58).

---

## DITCHED — With Brief Cause

### grid-rebuild-gridfix-marker (+1.217 ms wall, +0.364 ms phase — triage false positive)
Triage (500k) looked promising (dPhase -0.237 ms, dWall -2.755 ms) but full-pop (1M, 5 seeds) flipped to a clear regression above noise on both metrics. Adding lifecycle systems to maintain a `GridFixed` ZST marker (`sync_grid_fixed_removals`, `tag_stopped_catatonics`, `untag_on_behavior_change`) plus a cache `Vec` and `retain()` scan costs more than skipping the occasional catatonic entity. The stopped-catatonic cohort is too thin at 1M realistic-dna to offset the constant overhead of marker bookkeeping.

### perception-radius-prefilter (+0.096 ms, triage)
`cbrt()` is a transcendental (~30 cycles) called once per creature outside the inner scan. With realistic-dna population mix most proxies are not extremely small, so the prefilter rarely fires — constant cost, near-zero rejection savings. Benefit would only appear in heavily size-polarised populations the benchmark does not produce.

### motion-camouflage-freeze-skip (+0.048 ms phase, triage)
Two FP multiplies and a compare added per proxy to detect near-stationarity. With realistic-dna the vast majority of creatures are actively wandering; the frozen cohort is too small to recover the per-proxy overhead. Matches the failure mode predicted in `biology-notes.md`. Trophic canary was never reached.

### avoidance-skip-catatonic (+1.921 ms at 1M — triage false positive)
Triage showed -1.254 ms which triggered escalation to 1M. At 1M the signal reversed cleanly within noise. The inactive/catatonic fraction in realistic-dna is too small to produce measurable savings. Logically sound (movement zeros their accel anyway) but economically invisible.

### collect-cells-fov-squared-dist (-0.052 ms phase, noise floor)
Only ~10 cells per creature are processed here; the saved `sqrt` is unmeasurable at the per-creature level. The restructured three-branch comparison introduces extra branching correlating with worstPhaseP99Regression +0.492 ms. Wall delta -0.049 ms vs noise 1.139 ms is indistinguishable from noise.

### perception-bounded-k-min-buffer (+0.189 ms phase, triage)
The O(K) worst-slot linear scan over 7 elements per insertion exceeds the cost of the lazy partial-sort it replaces. Win only materialises when accepted candidates >> K, which does not happen in the typical per-perceiver cell scan at this density.

### hunger-gated-l1-range-halving (+1.156 ms wall, triage)
Energy branch inside the hot parallel L1 scan adds branch-prediction pressure on the ~95%+ of entities not in the hungry cohort. Bench population does not drain energy to the low-energy threshold fast enough during the measurement window. Same pattern as ledger #17 (sated cohort too thin).

### steering-local-cse-scalars (+1.125 ms wall, triage)
LLVM under `-O3` already CSEs `mass()`, `max_force()`, `max_speed()` since they are small fully-inlined methods. Consistent with ledger #36 and #55. Steering is memory/bandwidth-bound at 1M, not arithmetic-bound. Suggest DO_NOT_REVISIT for pure CSE micro-ops in the steering closure.

### l1-aggregation-l0-sort-by-parent (+14.832 ms wall, +15.027 ms phase — massive regression, stopped at triage)
`sort_unstable_by_key` over all non-empty L0 cells is O(N log N) ~ 3.4M comparisons at 500k pop. This single sort turns a ~3.4 ms phase into a ~19 ms phase. The L1 data path is memory-bandwidth-bound, not CPU-compute-bound — structural transforms that add CPU work to save cache bandwidth do not help. Same lesson as ledger #9 (parallel L1 aggregation). Do not revisit sort-based L1 locality approaches.

### l1-cone-fast-inv-sqrt (-2.274 ms raw p99, median sub-noise)
Raw p99 numbers look attractive but the median-level gate correctly rejects: dPhaseMedian +154 us vs phaseNoise 214 us; dWallMedian -405 us vs wallNoise 727 us. The L1 cone scan is low-frequency (only `CanStrategicVision` giants, only cells passing range+FOV). The compiler already lowers `recip()` to an efficient reciprocal sequence. Consistent with ledger #2.

---

## Bundle

No bundle A/B run. The only KEEP (`behavior-compact-active-set`) and the only DEFER (`fuse-behavior-transition-into-steering`) are **mutually exclusive** — both modify `behavior_transition_system` and a stacked union is incoherent. Pick one.

---

## Recommend Merging

| Priority | ID | Justification |
|----------|----|---------------|
| 1 | **behavior-compact-active-set** | -4.882 ms wall (replicated -4.951 ms), 30x phase SNR, no biology change, no hold, no canary required. Ready to land. |

**Fuse revisit:** If machine noise settles (baseline std < 1 ms), re-run `fuse-behavior-transition-into-steering`. The directional signal is -5.681 ms median / ~7.7 ms raw at 1M. If confirmed KEEP it supersedes behavior-compact-active-set (pick the fuse, drop the compact filter). These are mutually exclusive — do not merge both.

---

## What to Hunt Next

1. **Re-measure fuse-behavior-transition-into-steering** when baseline variance is low. Potential ~7.7 ms. Combined with native-par-iter (#58 HOLD, -7.258 ms) that would be ~15 ms in barrier-elimination wins alone, bringing 1M from ~35.5 ms toward the 20 Hz budget.

2. **Land held KEEPs from prior runs** before the next hunt — the merge queue is accumulating against a stale baseline:
   - `native-par-iter-kill-1m-collect` (#58, -7.258 ms, HUMAN HOLD)
   - `fuse-steering-integrate-system` (#33, -2.7 ms, HUMAN HOLD)
   - `behavior-compact-active-set` (this run, -4.882 ms, ready)
   Running the perf-hunt off an un-landed baseline understates cumulative improvement and produces misleading SNR numbers.

3. **Cohort-thin biological ideas** (hunger-gated, camouflage, avoidance-skip): all three failed because the target cohort is below ~5% of population at benchmark time. These may revive once the simulation runs long enough to drain energy and produce a realistic hungry/stopped/exhausted mix. Consider a warm-soak benchmark mode (run N ticks before measuring) to activate these cohorts.

4. **L1 aggregation** has resisted every structural attack (sort, parallel, fold — all DITCH). The next viable angle is hardware counter measurement via `dev-tools` perf events to determine if the bottleneck is DRAM bandwidth or something else before theorizing further.
