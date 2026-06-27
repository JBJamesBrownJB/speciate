# Phase 0 Run Summary — 2026-06-27

Baseline: wall p99 **34.5 ms** @ 1 M creatures, Windows, branch `perf/system-fusion`
(post `behavior-compact-active-set` merge on main).

Goal: extract each act-corridor system's per-creature body into a pure `step()` function.
No scheduling change, no semantic change. Confirm no regression at 1M/5-seed bench lab.

---

## Results

| System | Tests added | dWall p99 (ms) | dPhase (ms) | Committed |
|---|---|---|---|---|
| behavior_transition | 11 | +0.049 | −0.010 | yes — 23e1bc8 |
| steering | 6 | +0.602 | +1.048 | no — reverted |
| integrate_motion | 16 | −0.247 | −0.353 | yes — e6c7115 |
| perception | 0 | — | — | no — skipped at extract |

**2 of 4 committed. +27 new unit tests on the pure step() functions.**

---

## Per-system notes

**behavior_transition (committed)**
Extracted per-creature body into `transitions/step.rs`; `BehaviorStepCtx` holds `current_time: f64`.
System is a thin wrapper: build ctx once, same throttle/compact-collect/par_iter_mut(256) shape.
dWall +0.049 ms at 1M — within the 128 µs noise floor. Both the 2ms p99 gate and the 2x-noise
wall-median gate passed. This is a behavior-preserving refactor, not a perf win; the result confirms
the extraction introduced no overhead.

**steering (reverted)**
Extracted per-creature body into `steering/step.rs`; `SteeringStepCtx` hoists wander/seek force
multipliers from per-entity atomic reads. 500k triage failed the wall-median gate immediately:
dWallMedian 196 µs vs noise 46 µs (4.3× threshold). Steering phase p99 regressed +1049 µs
(3080→4129 µs). Hard-reverted with `git reset --hard && git clean -fd`. Step.rs removed.
Suspected cause: the step() signature takes 9 component borrows; the function-call overhead across
that borrow list is measurable at this population. Requires investigation before retry.

**integrate_motion (committed)**
Extracted per-creature body into `movement/step.rs`; `IntegrateCtx` holds dt, drag, noise scalars,
bounds, &NoiseTable. 16 tests assert exact mutated values across drag integration, speed clamping,
all four walls, rotation, and turn-rate. Full-pop (1M/5 seeds): dWall −0.247 ms, worst p99
regression 0.249 ms — both gates pass. Marginal improvement, not the goal; confirms no regression.

**perception (skipped at extract)**
The per-creature body is the spatial gather itself — every line is a grid API call or coordinate math
derived from grid queries. Three structural blockers made a pure step() unnatural:
1. Ctx would need &SpatialGrid + &CoarseGrid — large structures, not Copy scalars, violating the
   seam convention.
2. Dev-tools capture (debug_queried / debug_skipped Vecs) is built cell-by-cell inside nested
   iteration with no clean extraction point.
3. Thread-locals CELL_SCRATCH and NEIGHBOR_CANDIDATES are accessed via .with() inside the body —
   not truly isolatable for unit testing.
Working tree is pristine. Existing grid integration tests (tests.rs lines 462-556) already cover
the spatial behavior directly. Perception attack vector is smarter queries, not step() extraction.

---

## Phase-0 status

2 of 4 systems extracted and committed. The refactor is behavior-preserving in all committed cases —
the goal was hygiene and testability, not a wall-time win, and no regression was introduced.

Perception is correctly excluded from the fused path. Steering needs a targeted investigation before
retry (likely: `#[inline(always)]`, reducing the borrow-count in the step signature, or accepting
that steering stays as an unfused pass).

## Next

- **Phase 1:** Introduce `fuse-act` cargo feature. Initial fused path covers behavior + integrate
  only (perception excluded, steering pending re-evaluation). Single `par_iter` calling
  `behavior::step → integrate::step` under one `time("act")` scope.
- **Steering retry:** Profile the 9-borrow call overhead. Try `#[inline(always)]` on step(); if
  that resolves the regression, re-extract and re-bench.
- **Perception:** Attack with smarter spatial queries (FOV culling, range gating), not step()
  extraction.
