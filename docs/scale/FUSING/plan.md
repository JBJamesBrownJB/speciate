# 🚧 System Fusion — Master Plan

> Living tracker for the staged refactor toward a fused per-creature "act" pass.
> Branch: `perf/system-fusion`. Update the worklist + progress log as we go.

## Goal

Collapse the per-creature **act corridor** — `perception → behavior → steering → integrate_motion` —
from four separate Bevy systems into one fused per-creature pass **at ship time**, while keeping the
systems **separate and observable during development**. The refactor that makes this safe and cheap
(Phase 0) lands now; the fused execution path and its proof land in later phases.

## Why it's worth it

At 1M creatures the wall is **memory bandwidth + fork-join barriers**, not compute. The four act
systems each sweep the *same* 1M creatures, with a fork-join barrier between each. Fusing the chain
into one closure removes the redundant cache re-sweeps and the barriers. Measured potential:
`behavior→steering` alone = **−5.7 ms** wall p99 (ledger DEFER 2026-06-27); the full corridor stacks
higher. The biggest collect-killer (`native-par-iter`, −7.26 ms) is **already merged** (2026-06-26).

## The two concerns — and why we proceed anyway

1. **Coupling.** Mitigated by extracting each concern into a **pure `step()` function in its own
   file** (single responsibility). The fused system becomes a thin orchestrator that calls them in
   sequence inside one `par_iter`. Residual cost: the fused system's query widens to the *union* of
   the concerns' component access — but the chain is already strictly sequential, so almost no real
   parallelism is lost.
2. **Lost observability.** Per-system sparklines come from RAII scope timers
   (`instrumentation/mod.rs:153`, `time(name) -> TimingGuard` → per-system `AtomicU64`). Fusion
   removes the scope boundary, so the concerns can't be timed separately *in one run* (per-entity
   timing = atomic contention across rayon workers; splitting the loop = un-fusing). **Solved by a
   dual compile path:** dev build = separate + timed, prod build = fused, both calling the *same*
   pure `step()` functions, with a bit-identical guard test.

## Architecture facts (anchors)

- **Schedule:** `apps/simulation/src/simulation/core/simulation.rs:82` (the `add_systems` chain).
- **Timing:** `apps/simulation/src/instrumentation/mod.rs:153` (`time(name)` RAII guard; granularity
  = the timer scope, which is why fusion erases the per-concern split).
- **The corridor systems:**
  - perception — `apps/simulation/src/simulation/perception/systems.rs`
  - behavior — `apps/simulation/src/simulation/creatures/behaviors/transitions/systems.rs`
  - steering — `apps/simulation/src/simulation/creatures/steering/system.rs`
  - integrate — `apps/simulation/src/simulation/movement/systems.rs`
- **Why `.after()`:** producer→consumer data dependency on shared creature components (perception
  needs the rebuilt grid; steering needs behavior's mode + perception's neighbors; integrate needs
  steering's force). Bevy serializes them regardless of explicit ordering — running them at once
  would be a data race. Parallelism is **within** each system (`par_iter` across creatures), not
  across systems. Fusion removes the **barrier between** phases, it does not parallelize the chain.

## What does NOT fuse (proven — do not retry)

- **grid_rebuild + l1_aggregation** — DITCHED (`fuse-l1-into-rebuild`, +0.034 ms = nothing). Shared
  grid-cell scatter/gather, a different parallelization model.
- **parallel-l1-aggregation** — DITCHED (+3 ms, worse).
- **update_plants** — different entity set (plants); cannot fuse with the creature chain.
- **perception's intrinsic cost** is the neighbor **gather** (algorithmic), not a barrier — fusion
  saves little there. Attack perception with smarter queries, not fusion.

---

## Phases

### Phase 0 — Decouple into pure functions (NOW · pure-upside hygiene)

Extract each concern's per-creature logic into a pure `step()` function in its own module. Each
system becomes a thin wrapper: open timer scope → `par_iter` → call the one `step()`. **No
scheduling change, no behavior change, sparklines untouched.** Build/expand TDD around each pure
function as it's extracted (they're now testable in isolation — a direct win).

Worklist — **one system per commit**, in dependency order:

- [x] **behavior_transition → `behavior::step`** — beachhead; sets the seam + test pattern
- [x] **steering → `steering::step`** — retried with `#[inline(always)]` convention; dWall −0.364 ms, dSteer −0.196 ms; both reps all gates passed; committed e19bc11
- [x] **integrate_motion → `integrate::step`**
- [ ] **perception → `perception::step`** (if cleanly separable from the gather) (skipped: spatial-gather coupling — every line is a grid API call, no separable logical core)
- [ ] incidental per-creature helpers (`update_body_size_cache`, etc.) as encountered

**Per-step protocol (every system):**
1. Write/expand the pure-function unit tests **first** (red), capturing the contract.
2. Extract the pure `step()`; make the system a thin wrapper. Behavior-preserving.
3. Full `cargo test` green.
4. Run the **full bench lab** (`latency_lab`, 1M, 5 seeds) A/B vs the pre-step baseline — confirm
   **no regression**. Record the number in the progress log below.
5. Commit (small, reviewable). Move to the next system.

#### Phase 0 close-out (added 2026-06-27 — RESOLVED 2026-06-27, see `phase0-closeout.md`)

All exit gates passed. Full results in `docs/scale/FUSING/phase0-closeout.md`.

- [x] **Cumulative branch-vs-main lab A/B (NEW required gate).** The run bench-tested each extraction
   *independently* vs baseline — it never validated the **whole stack together** vs `main`. Live
   snapshots then showed a *stable* ~+2.5 ms offset in the **steering** phase on the branch (source
   byte-identical to main) across two captures — signal-shaped, not noise. Likely a **code-layout /
   i-cache artifact** of adding the new `step` modules (only visible with all changes present). Run a
   same-session, back-to-back, 5-seed `latency_lab` A/B of **branch HEAD vs main**, replicated, to
   resolve real-vs-noise. This is the Phase 0 exit gate; nothing proceeds to Phase 1 until it's clean.
   **RESOLVED 2026-06-27:** `steering_offset_real = false`. Replicated A/B (4 runs, pop=1M, 5 seeds)
   — steering delta negative in all reps (candidate faster); mean dWall −0.755 ms; VERDICT=Defer
   (improvements present, no regression in any gate).
- [x] **Inline convention.** "Extraction is free" only holds when `step()` is force-inlined — the
   steering regression proved it. Make **`#[inline(always)]`** a uniform rule and apply it to the
   committed `behavior::step` (currently none) and `integrate::step` (currently `#[inline]`). Re-bench
   vs main; test whether inlining also closes the steering layout offset (fewer separate functions →
   less layout perturbation).
   **RESOLVED 2026-06-27:** `#[inline(always)]` applied uniformly to all three `step()` functions;
   harden commit 63a6e43. Steering offset resolved in both harden reps; dWall −0.91 ms.
- [x] **DSL unification.** The two extractions diverged (`BehaviorStepCtx` vs `IntegrateCtx`; pure
   scalars vs a `&NoiseTable` borrow). Pick ONE convention — `<Concept>Ctx` naming, and relax the
   overstated "Ctx = Copy scalars only" rule to **"scalars + read-only borrows"** (integrate already
   needs a borrow; that's fine). Behavior-preserving rename, tests stay green.
   **RESOLVED 2026-06-27:** `BehaviorCtx` / `IntegrateCtx` / `SteeringCtx` — uniform `<Concept>Ctx`
   naming; "scalars + read-only borrows" convention. Applied in harden + steering commits.

Then the original outstanding item: **steering retry** — re-extract `steering::step` *with* the
inline convention from the start; on the wide-borrow regression, fall back to a reduced/struct borrow
signature. If it still regresses, document steering as a justified exception (stays a separate system,
like perception) rather than forcing it. A GOOD Phase 0 outcome = the corridor decoupled **as far as
it cleanly can be**, hardened, validated cumulatively vs main, and documented — not necessarily all 4.
**RESOLVED 2026-06-27:** Extracted and committed e19bc11 via `inline-always` approach. dWall −0.364
ms, dSteer −0.196 ms; both reps passed all three gates.

### Phase 1 — Fused path behind a flag — ✅ DONE 2026-06-28

- Add cargo feature **`fuse-act`**, **orthogonal to `dev-tools`** (so the fused build stays
  measurable — this is what lets us benchmark prod).
- `#[cfg(feature = "fuse-act")]` registers one `act_system` that `par_iter`s once and calls
  `behavior::step → steering::step → integrate::step` inline, under a single `time("act")` scope.
  `#[cfg(not(...))]` keeps today's separate systems.

**Delivered** (`apps/simulation/src/simulation/act/mod.rs`): `fuse-act = []` feature (default off,
no deps); fused `act_system` reproducing each step's per-entity gate EXACTLY — behavior throttle
(`behavior_divisor` bucket), steering Dormant-skip + capability bools, unconditional integrate — so
the (entity × step) execution set is identical to the unfused schedule. `act_us` timing metric added
to `SystemTimings`/snapshot; loop wrapped in `time("act")` under dev-tools. Steering `debug_assert!(accel==0)`
carry-in added (both paths). Schedule cfg-gated (exactly one path). Build wiring: `dev:release` → fused
(`build:fused` = `--features dev-tools,napi,fuse-act`); new `dev:release:unfused` → separated;
`package`/`build`/`build:debug`/`latency_lab` left UNFUSED.

**Verified:** TDD `fused_act_corridor_moves_seeker`; full suite green both configs (unfused 564, fused 565)
across 26 fused + 8 unfused runs; all build configs compile; ecs-emma adversarial review = behavior-preserving,
no entity gains/loses a step. Left uncommitted for human gate, then committed to `perf/system-fusion`.

**Known follow-ups (Phase 2 / hygiene), NOT blockers:**
- **Dev-UI force-vector overlay goes stale in fused+dev-tools builds.** `capture_debug_acceleration_system`
  ran *between* steering and integrate to snapshot the inspected creature's capped accel; there is no
  observation point mid-fused-loop, so it's cfg'd out under `fuse-act`. Sim state identical (it was a
  read-only debug capture); `capture_debug_accel_us` reads 0 fused. Use `dev:release:unfused` to inspect
  forces. Accepted tradeoff for the fused (perf/ship) path.
- **`act` sparkline not yet shown in dev-UI.** `act_us` is captured but the dev-UI reads a fixed phase
  set; add `actUs` to its sparkline list so fused dev builds show the single `act` bar (today they'd show
  behavior/steering/movement as 0). Small frontend follow-up.
- **One rare pre-existing flaky test** surfaced once in ~35 runs (a `thread_rng`-class test, same RNG in
  both paths; could not reproduce in 26 fused retries). Not fusion-introduced; track + harden separately.

### Phase 2 — Prove it + guard it — ✅ DONE 2026-06-28

- **Bit-identical guard ✅.** `fused_and_separate_act_are_bit_identical`
  (`core/simulation.rs`, `#[cfg(feature="fuse-act")]`): a seeker+dormant population run 30 ticks
  through the fused schedule vs the **real** separate schedule, asserting exact
  Position/Velocity/Rotation equality. To compare the two compile-time paths in one binary, the
  schedule wiring was extracted into named `add_separate_act_systems` / `add_fused_act_systems`
  fns (DRY; the guard uses the genuine separate wiring, not a copy that could drift) + a test-only
  `with_separate_act_schedule()`. Determinism is engineered: noise off, rotation pinned via
  `facing_direction` (spawn otherwise draws `thread_rng`), seeker-only population (the wander
  branch is the one irreducible `thread_rng`, identical in both paths anyway). **Proven to bite** —
  mutating the fused arrival→Catatonic turned it red precisely on the arriving seekers, green on
  revert.
  - *Bonus:* fixed a **pre-existing** dev-tools test bug surfaced while running the full matrix —
    `update_plants_runs_without_panicking` never inserted `SystemTimings` (Phase 1 only ran the
    non-dev-tools suites, 564/565). Confirmed pre-existing via `git stash`. Full matrix now green:
    **default 564 / fuse-act 566 / dev-tools 673 / dev-tools,fuse-act 675.**

- **Benchmark ✅ — fused is faster, replicated.** `latency_lab` A/B, two builds of the same bin
  (`--release --features dev-tools` vs `…,fuse-act`), pop=1M, seeds 11/42/99/137/2025 (Phase 0's
  set), two same-session back-to-back reps. (Raw `latency_lab` MultiSeedReport JSON kept locally —
  perf blobs aren't committed; the durable numbers are the table below.)

  | Rep | sep wall p99 | fused wall p99 | dWall p99 | sep wall mean | fused wall mean | dWall mean | verdict dWallMedian / wallNoise |
  |-----|-----|-----|-----|-----|-----|-----|-----|
  | R1 | 49.54 ms | 44.29 ms | **−5.25 ms** | 42.99 ms | 38.62 ms | **−4.36 ms** | −4.31 ms / 3.54 ms (~1.2σ) |
  | R2 | 44.38 ms | 41.36 ms | **−3.02 ms** | 39.26 ms | 36.57 ms | **−2.68 ms** | −2.73 ms / 0.63 ms (**~4.4σ**) |

  Fused faster on **all three** wall metrics in **both** reps. `cells_queried` identical
  (~5.927 M both reps) ⇒ **identical work** — the drop is pure barrier + cache-re-sweep removal,
  not less computation. Control phase (perception, untouched by fusion) flat (R1 +155 µs within
  1474; R2 +829 µs drift — and wall *still* netted −2.7 ms, so the act saving is *larger* than the
  raw wall delta). **Conservative headline: ~3–5 ms wall p99 saved at 1M (≈6–10% of tick), the
  predicted magnitude from removing 2 fork-join barriers across the ~23.7 ms corridor.**
  - **Caveat — the `classify()` VERDICT reads `Ditch`, which is a gate artifact, not a negative.**
    `classify()` is *phase-targeted* (it asks "did the named phase improve?"). Fusion's target is the
    `act` phase, which the lab's fixed phase-set doesn't track; run with `--phase perception` (the
    control), perception correctly *doesn't* improve → `Ditch`. The honest signal is `dWallMedian`,
    which is solidly negative both reps. **Follow-up:** teach the lab's per-phase tracker about `act`
    so a future fused A/B can gate on the act-phase delta directly instead of leaning on wall.

- **Result:** faster **and** bit-identical → the ship build *should* run `fuse-act` with no
  `dev-tools` (`--release --features napi,fuse-act`); the dev build already runs fused via
  `dev:release`. **Build wiring DEFERRED to ship time (JB, 2026-06-28):** the production `package`
  path currently has no coherent ship config (ships un-fused *and* with dev-tools) — logged as a
  standalone item in `docs/technical-debt.md` §"Production / Ship Build", to be sorted when we
  actually package a release. `latency_lab` stays a dual-build A/B tool regardless.

---

## Discipline (non-negotiable)

- **Test-FIRST** around every pure function (the extraction is the moment to strengthen coverage).
- **Full bench lab after every system extraction** — no batching; each step proves zero regression
  on its own, with the number logged here.
- **One system per commit**, kept reviewable.
- **This doc is the source of truth** for progress — keep the worklist checkboxes and the log current.

## Progress log

- **2026-06-27 (init)** — Branch `perf/system-fusion` + this plan created. Next action: capture a fresh
  `latency_lab` baseline (post `behavior-compact-active-set` merge, which is on `main`) to anchor
  Phase 0, then start the `behavior_transition` beachhead extraction.

- **2026-06-27 (Phase 0 run)** — Baseline wall p99 34.5 ms @ 1M. Phase 0 complete: 2 of 4 systems
  committed; 1 failed bench gate; 1 skipped at extract.
  - **behavior_transition** — extracted to `transitions/step.rs`; 11 tests added; dWall p99 +0.049 ms
    (within 128 µs noise floor); dPhase −0.010 ms; **committed** 23e1bc8.
  - **steering** — extracted to `steering/step.rs`; 6 tests added; dWall p99 +0.602 ms (wall-median
    196 µs vs noise 46 µs = 4.3× threshold); dPhase +1.048 ms; **reverted**, step.rs removed. Root
    cause unknown — likely function-call boundary cost across the wide borrow set (9 components).
  - **integrate_motion** — extracted to `movement/step.rs`; 16 tests added; dWall p99 −0.247 ms;
    dPhase −0.353 ms; **committed** e6c7115.
  - **perception** — skipped at extract stage; no files modified. Blockers: Ctx would need &SpatialGrid
    + &CoarseGrid (violates Copy-scalar convention), dev-tools capture is structurally embedded in
    nested cell iteration, and thread-local CELL_SCRATCH / NEIGHBOR_CANDIDATES prevent true isolation.
  - **Next:** (a) Investigate steering regression — profile call overhead vs inlining; consider
    `#[inline(always)]` or collapsing the borrow list. (b) Phase 1 (`fuse-act` feature flag) is
    unblocked for behavior + integrate; perception is excluded from the initial fused path. (c)
    Perception attack vector remains smarter queries (FOV culling, range gating), not step() extraction.

- **2026-06-27 (Phase 0 close-out)** — All Phase 0 exit gates cleared. Replicated A/B confirmed
  `steering_offset_real = false` (prior session signal was thermal/session noise, not i-cache layout).
  Harden pass applied `#[inline(always)]` uniformly and unified DSL to `<Concept>Ctx` (commit
  63a6e43; dWall −0.91 ms, steering offset resolved both reps). Steering retried with
  `inline-always` approach: extracted to `steering/step.rs`, 6 tests, dWall −0.364 ms, dSteer −0.196
  ms — committed e19bc11, both reps all gates passed. Final cumulative verify (4 A/B runs, 1M/5
  seeds): worst-rep dWall −0.816 ms, worst-rep dSteer −0.179 ms, VERDICT=Defer (improvements
  sub-threshold but no regression in any run). Branch clean vs main. All three act-corridor systems
  have pure `step()` functions. Perception remains a justified exception (spatial-gather coupling).
  **Phase 0 DONE. Next: Phase 1 — `fuse-act` feature flag.**

- **2026-06-28 (Phase 1)** — `fuse-act` feature + fused `act_system` landed (scout→implement→verify
  workflow). Per-entity gating reproduced exactly (behavior throttle / steering Dormant-skip / uncond.
  integrate); `act_us` metric + `time("act")`; steering `debug_assert` carry-in. Wiring flipped per JB:
  **`dev:release` = fused**, `dev:release:unfused` = separated, package + lab stay unfused. Suite green
  both configs (564 unfused / 565 fused, 26+8 runs); ecs-emma review = behavior-preserving. See Phase 1
  follow-ups above (dev-UI force overlay stale fused; `act` sparkline display; one rare pre-existing flake).
  **Phase 1 DONE. Next: Phase 2 — bit-identical guard test + `latency_lab` fused-vs-unfused A/B (the proof).**

- **2026-06-28 (Phase 2)** — Both halves landed. (a) **Bit-identical guard**: schedule wiring
  extracted to `add_separate_act_systems`/`add_fused_act_systems` + test-only
  `with_separate_act_schedule()`; `fused_and_separate_act_are_bit_identical` proves fused ≡ separate
  to the bit (exact Position/Velocity/Rotation over 30 ticks), and was mutation-tested to confirm it
  bites. Fixed a pre-existing dev-tools-only flake (`update_plants` test missing `SystemTimings`).
  Full matrix green: 564 / 566 / 673 / 675. (b) **A/B benchmark** (1M, 5 seeds, 2 reps): fused faster
  on every wall metric both reps — dWall p99 −5.25 / −3.02 ms, dWall mean −4.36 / −2.68 ms; identical
  `cells_queried` (same work); perception control flat. `classify()` `Ditch` is a phase-gate artifact
  (perception=control didn't improve; the real proof is the negative `dWallMedian`).
  **Phase 2 DONE.** Ship-build wiring (flip `package` to `napi,fuse-act`, no dev-tools)
  **deferred to ship time** — logged in `docs/technical-debt.md` §"Production / Ship Build". Optional
  later: track `act` as a lab phase so fused A/Bs gate on the act delta, not wall.
