# Phase 0 Close-out — 2026-06-27

Branch: `perf/system-fusion`. Clears the exit gates from the Phase 0 run so Phase 1 can begin.

---

## Steering Offset Diagnosis

The prior Phase 0 session left a live-capture signal: a stable ~+2.5 ms offset in the steering phase
on the branch vs main, despite byte-identical steering source. Two hypotheses: (a) real i-cache layout
perturbation from adding new `step` modules, (b) thermal / session noise.

**Result: noise. `steering_offset_real = false`.**

Replicated back-to-back A/B (branch HEAD vs main, pop=1M, seeds 11/42/99/137/2025, clean-tree guard
passed in both reps):

| Rep | dSteerMedian | phaseNoise | Signal? |
|-----|-------------|-----------|---------|
| R1 | −7 µs | ±117 µs | No — candidate faster, within noise |
| R2 | −167 µs | ±220 µs | No — candidate faster, within noise |

Neither rep shows steering crossing into positive territory. The prior-session signal was thermal /
session noise. The branch was clean vs main before the harden pass.

The harden pass (`#[inline(always)]` uniformly applied, commit 63a6e43) further confirms this:
steering offset resolved in both harden reps, ruling out any residual layout concern from the inline
delta.

---

## Close-out Change Table

| Change | dWall p99 (ms) | dSteer (ms) | Committed |
|---|---|---|---|
| Harden: `#[inline(always)]` uniform + DSL unification | −0.91 | +0.048 (noise) | yes — 63a6e43 |
| Steering: `steering::step` extracted, `inline-always` | −0.364 | −0.196 | yes — e19bc11 |
| Cumulative verify (4 A/B runs, pop=1M, 5 seeds) | −0.816 (worst rep) | −0.179 (worst rep) | n/a |

dWall and dSteer are candidate-minus-main; negative = candidate faster. The verify runs emitted
VERDICT=Defer (exit 2): improvements present but sub-threshold for a confirmed-win signal. No
regression in any of the three gates (wall median, steering phase, worst p99 per-phase) across all
four runs.

---

## Final Cumulative Verdict

**Branch `perf/system-fusion` is clean vs main.**

Full Phase 0 worklist final state:

| System | Tests added | dWall p99 (ms) | Committed |
|---|---|---|---|
| behavior_transition | 11 | +0.049 (noise) | yes — 23e1bc8 |
| integrate_motion | 16 | −0.247 | yes — e6c7115 |
| steering | 6 | −0.364 | yes — e19bc11 (close-out) |
| perception | 0 | — | no — justified exception |

33 new unit tests across the three extracted `step()` functions. All three act-corridor systems are
now independently testable pure functions. Perception excluded (spatial-gather coupling — every line
is a grid API call; thread-locals CELL_SCRATCH / NEIGHBOR_CANDIDATES prevent true isolation).

---

## Adversarial Review Findings

The code review assessed all three extractions as behavior-preserving, DSL-consistent, and
test-strong. Four residual items were flagged:

**1. Mis-staged artifact (concrete, cosmetic)**
`docs/performance/snapshots/win_pop1M_33.2ms_2026-06-27_1958.json` (504 lines) was swept into commit
23e1bc8 ("refactor(behavior): extract pure step()"). It is an unrelated perf-capture artifact that
was untracked at session start. No behavior impact. Options: accept as-is or revert the file from
that commit in a follow-up. Low priority.

**2. Lost biological rationale (concrete, low-risk)**
`movement/step.rs` dropped the `movement/systems.rs:157` comment: "Biological basis: turn_rate ∝
1/size^1.33 — moment of inertia vs muscle torque". Per project policy (AGENTS.md), comments are a
code smell; this note belongs in `docs/biology/biology-notes.md`. The zero-cross rationale survives
as the `step_normal_preserves_rotation_when_speed_zero_crosses_below_threshold` test. Action: migrate
the biology note to the docs at a convenient time.

**3. Steering cap precondition (latent risk — add assert in Phase 1)**
`steering::step()` now caps steering-only accumulated forces; the wrapper then does
`acceleration.ax += output.ax`. This is semantically equivalent to the original (cap of the
post-accumulation Acceleration component) only because Acceleration is guaranteed zero at steering
entry. The invariant holds today: schedule is perception→behavior→steering→integrate, integrate clears
Acceleration at tick end, and the one system between steering and integrate (`capture_debug_acceleration_system`,
dev-tools only) is a read-only path. If any future system writes Acceleration before steering, cap
semantics diverge silently. Add `debug_assert!(acceleration.ax == 0.0 && acceleration.ay == 0.0)` at
`steering/step.rs` entry in Phase 1.

**4. Lost throttle rationale (low-risk)**
`behaviors/transitions/systems.rs` dropped the WHY comment explaining the compact-before-dispatch
ordering. The behavior is guarded by the `test_compact_active_set_cadence_preserved` test; the
rationale has no current home. Worth a sentence in `docs/scale/` or a named constant.

---

## PHASE 0 STATUS: DONE

All exit gates cleared. The act corridor is decoupled into pure `step()` functions — uniformly
`#[inline(always)]`, with a shared `<Concept>Ctx` DSL convention, validated cumulatively vs main,
and covered by 33 new unit tests.

**Next: Phase 1 — `fuse-act` cargo feature flag.**

Fused corridor: `behavior::step → steering::step → integrate::step` in one `par_iter` under a single
`time("act")` scope. Perception excluded. Bit-identical guard test is a Phase 2 deliverable (the
safety net that lets the two compile paths diverge without silent semantic drift).

**Residual risks entering Phase 1:**

- Steering cap precondition: invariant held by schedule ordering, not asserted — add `debug_assert`
  in Phase 1 before any force-writing system is added to the corridor.
- Mis-staged JSON snapshot in 23e1bc8: cosmetic, no behavior impact, low priority to clean up.
