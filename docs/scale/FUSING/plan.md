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
- [ ] **steering → `steering::step`** (failed: wall-median regressed 4.3× noise at 500k triage; reverted)
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

### Phase 1 — Fused path behind a flag (later · pre-ship)

- Add cargo feature **`fuse-act`**, **orthogonal to `dev-tools`** (so the fused build stays
  measurable — this is what lets us benchmark prod).
- `#[cfg(feature = "fuse-act")]` registers one `act_system` that `par_iter`s once and calls
  `behavior::step → steering::step → integrate::step` inline, under a single `time("act")` scope.
  `#[cfg(not(...))]` keeps today's separate systems.

### Phase 2 — Prove it + guard it

- **Bit-identical test:** run N ticks separate vs fused from one seed; assert identical world state
  (positions/velocities). The safety guard against the two paths silently diverging.
- **Benchmark:** `latency_lab` A/B —
  - `--release --features dev-tools` → separate path, wall p99
  - `--release --features dev-tools,fuse-act` → fused path, wall p99
  
  The delta is the proof the prod build is faster. Reuses the perf-hunt A/B machinery.
- If faster **and** bit-identical → ship build enables `fuse-act`; dev build does not.

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
