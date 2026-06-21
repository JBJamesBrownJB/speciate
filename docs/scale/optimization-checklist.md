# One-Million Optimization Checklist 🚧

> **Category: 🚧 In progress (NOW) — Pillar 1.** The living try → test → keep/ditch tracker for
> pushing max sustainable population from ~920K (mean) toward **1,000,000**. Every item is
> validated in the [latency tuning lab](./latency-tuning-lab.md) against the verified baseline
> in [`path-to-one-million.md`](./path-to-one-million.md). Synthesised from a four-specialist
> analysis (ECS/DOD, Rust hot-loop, biology/Golden-Zone, instrumentation), 2026-06-21.

**Status legend:** ☐ TODO · 🔬 TESTING · ✅ KEEP · ❌ DITCH · ⏸ BLOCKED (needs a prerequisite)

**The gap to close:** at 900K the tick is 48.3 ms mean / **56.9 ms p99** (budget 50 ms). 1M is
~56 ms mean / 68 ms p99 — **~12% over.** The **p99 tail breaks before the mean**, so the tail is
the real scoreboard. Growth is ~O(n¹·¹⁵) (fixed-world density rises with population).

## Findings log (verdicts from real runs)

**2026-06-21 — Lab prereqs built** (multi-seed noise floor, p99-per-phase diff, cells_queried).
At 900K the wall-p99 **noise floor ≈ 1.9–2.4 ms** — the bar any single change must clearly beat.

- **T1.1 (exponent 0.35 → 0.25):** ✅ **KEEP for correctness · ⚠️ perf within noise.** Perception
  mean −8%, but wall p99 −1.1 ms (< noise floor) — not a confident perf win alone. Kept because it
  is biologically correct (eye allometry ~^0.25) and creates emergent prey-refuge (see
  `biology-notes.md`). Note: `cells_queried` barely moved — it counts **L0** cells, but the trim
  affects the **L1 cone scan** → we need an **L1-cells counter** (instrumentation gap).
- **T1.2 `fast_inv_sqrt` + T1.3 chunk 64→256:** ❌ **DITCH (reverted, `87022fa`).** Stacked with
  T1.1, wall p99 = 48.5 ms vs 48.0 ms baseline — flat/worse, within noise. Perception mean −6–8%
  but steering/movement/grid drifted up ~0.5 ms (Rayon phase-transfer). No budget movement.

> **META-LESSON: perception-mean micro-opts (<1 ms true effect) sit below the ~2 ms wall-p99 noise
> floor — they do not move the tick budget.** The p99 tail is set by **variance + fork-join
> barriers**, not by one phase's average. **Pivot:** chase the big-effect levers — **T3.1**
> (schedule overlap → fewer barriers, attacks tail variance directly) and **T2.2 / T2.5**
> (hunger-gating deletes whole perception ticks; stochastic phase smooths the tail). Stop
> micro-tuning Tier 1.

## The model (where the budget actually goes)

Perception (14.5 ms) + steering (13.1 ms) = **57% of the tick**, and phases sum to wall (no
serial idle between them) — so wins come from *within-phase parallelism* or *doing less work*.
The cost is **density-amplified**: each L0 cell holds more proxies as population rises, and the
**L1 cone scan is O(range²) per creature** with range varying ~100:1 across the random-DNA
population. A handful of large, narrow-FOV creatures (~10% of the pop, size ≥ 5 m) carry a
hugely disproportionate share of perception time and **own the p99 tail**. That makes
large-creature perception range the single highest-leverage lever — and it is also the
*biologically wrong* part of the current model (real detection range scales sub-linearly with
size and saturates from atmospheric/water extinction). Perf win = correctness win = the Golden Zone.

Verified constants (`apps/simulation/src/simulation/creatures/constants/perception.rs`):
`PERCEPTION_MULTIPLIER = 10`, `SIZE_ALLOMETRY_EXPONENT = 0.35`, `SIZE_ALLOMETRY_REFERENCE = 0.5`,
`FOV_RANGE_EXPONENT = 0.4`. (The comment in `perception/tests.rs:167` saying multiplier = 100 is
**stale** — fix while here.) Note: `perception.range` drives only the **L1** cone scan; the L0
9-cell scan is fixed by `L0_SCAN_RADIUS` (`perception/systems.rs:27`).

---

## Prerequisites — build these FIRST (results are unprovable without them)

Per the instrumentation analysis, the lab currently cannot detect the failure modes these
experiments risk. Do these before trusting any keep/ditch verdict.

- [ ] **L1 · p99-per-phase in `diff_reports`** — `bench_lab/report.rs:28` diffs `mean` only, so a
  "mean win / tail regression" is structurally invisible. Extend `PhaseDelta` with
  `before_p99_us/after_p99_us/delta_p99_us`. *(struct field + one line per phase)*
- [ ] **L2 · Multi-seed runner + across-seed noise floor** — `run_lab`/`find_max_pop` use a single
  seed; the noise floor at 900K is unknown, so a 1.5 ms delta may be noise. Add
  `run_lab_multi_seed(cfg, &[seed])` returning mean-of-means, mean-of-p99s, std-of-p99s. *(day)*
- [ ] **L3 · `cells_queried` / `neighbor_comparisons` counters in `PhaseSamples`** — converts
  attribution from correlative ("time dropped") to causal ("cells scanned dropped 40% and time
  tracked it"). Essential signal for the range-trim and skip levers. *(instrument perception +
  surface through `get_system_timings()`)*

---

## Tier 1 — cheap mechanical wins (low risk, do first)

| ☐ | Experiment | Target | Est. mean | Tail? | Effort |
|---|------------|--------|-----------|-------|--------|
| ☐ | **T1.1 Trim large-crit range** — lower `SIZE_ALLOMETRY_EXPONENT` 0.35 → 0.25 (try 0.20) | `constants/perception.rs:61` | **2–4 ms** | **Primary** | 1 line + test |
| ☐ | **T1.2 `fast_inv_sqrt` in L1 cone scan** — replace `dist_sq.sqrt().recip()` | `perception/systems.rs:545` (helper `math/vector_ops.rs:33`) | 0.5–1.5 ms | Yes | 1 line |
| ☐ | **T1.3 Tune perception `with_min_len`** 64 → sweep {128,192,256} | `perception/systems.rs:116` | 0.5–1.5 ms | Yes | 1 number |
| ☐ | **T1.4 Hoist `thread_rng` from wander** — per-entity seeded PRNG | `steering/system.rs:38` | 0.2–2 ms | Mild | afternoon |

- **T1.1** is the headline. Lowering the allometry exponent trims *only* large crits (small/medium
  untouched): size-10 narrow-FOV range −26%, L1 cells scanned −38–40%. Keep-criterion below is
  stricter than perf alone because it changes behaviour. Add a guard test
  (`Perception::new(10.0, 45.0).range < 400.0`) so it can't silently regress. Log the chosen
  value + biological rationale in `docs/biology/biology-notes.md`.
- **T1.4** bonus: a seeded-per-entity PRNG also makes wander deterministic (helps the lab track
  the engine). Verify wander stays isotropic (no directional bias).

---

## Tier 2 — biological Golden-Zone levers (perf **and** gameplay; need biology + DNA work)

Real ecology is **sparse-attention**: animals run cheap ambient monitoring and only spin up
expensive directed perception when hunger / threat / motion demands it. The current sim makes
*every creature a maximally-alert apex hunter every tick* — biologically false and the fattest
compute line. These levers delete work that *shouldn't exist*. They **multiply** (range cap
shrinks scan area; gating cuts tick count; skips prune candidates within each scan).

| ☐ | Lever | Mechanism / try-value | Load cut | Gameplay payoff |
|---|-------|----------------------|----------|-----------------|
| ☐ | **T2.1 Range cap** | `min(range, 400–600 m)` after fov_factor (pairs with T1.1) | Large (collapses outlier tail) | Prey gain real refuge distance; safe zones, stalking |
| ☐ | **T2.2 Hunger-gated frequency** | energy >70% → throttle 1/8, 40–70% → 1/4, <40% → every tick | Large (most creatures fed) | Post-meal safety windows; hungry = frantic |
| ☐ | **T2.3 Motion-detection skip** | ignore entities < ~0.5 m/s, near-field override (~1 body-length) | Medium–large | Freezing = real camouflage |
| ☐ | **T2.4 Size-domination skip** | predator ignores prey < ~2% own mass | Medium (hits large-crit lists) | Small-creature size-refuge niche |
| ☐ | **T2.5 Stochastic phase** | random per-creature perception phase offset + jitter | Medium — **best tail smoother** | Natural staggered reaction delays |
| ☐ | **T2.6 Size reaction-latency** | big crits re-decide every 4–8 ticks (steering cost) | Medium | You can juke a giant |
| ☐ | **T2.7 Terrain/cover range** (new) | cut range in dense cover/water | Medium (where density is highest) | Cover as a stealth mechanic |

> **Shared guardrail for ALL of Tier 2 — the canary:** every lever risks the same failure mode,
> *predator goes blind → starves → trophic cascade* (predator collapse → herbivore boom →
> vegetation crash). **KEEP only if perf improves AND no species goes extinct or explodes over a
> long run** (apex-predator and primary-grazer counts stay within ~±20% of pre-lever baseline).
> Watch those two populations as the canary. Log every chosen range/threshold in `biology-notes.md`.

Suggested order: **T2.1 → T2.2 → T2.3 → T2.4 → T2.5 → T2.6**.

---

## Tier 3 — structural (higher ceiling, higher risk, multi-day)

| ☐ | Experiment | Target | Est. mean | Tail? | Effort |
|---|------------|--------|-----------|-------|--------|
| ☐ | **T3.1 Schedule overlap** — break the 7-phase `.after()` chain; run grid rebuild + L1 (after `integrate_motion`) concurrently with behavior/steering; let Bevy parallelise disjoint `ResMut<Grid>` vs creature queries | `core/simulation.rs:75-93` | 3–5 ms | **Best** (fewer barriers) | multi-day |
| ☐ | **T3.2 Perception proxy SoA split** — hot `[x,y,radius]` column vs cold `[vx,vy,entity]`; denser reject scan | `spatial/grid.rs:1047`, `:382` + call sites | 1.5–3 ms (scales with density) | Yes | multi-day |
| ☐ | **T3.3 Kill per-tick `iter_mut().collect()`** — persistent/`par_iter_mut` for the 900K-element gather in perception/steering/movement/behavior (Windows page-fault tax) | `perception/systems.rs:99`, `steering/system.rs:164`, `movement/systems.rs:54`, `transitions/systems.rs:26` | 1–3 ms | Partial | day |
| ⏸ | **T3.4 Parallelise serial L1 aggregation** — `par_chunks` + per-thread partials | `spatial/hierarchical.rs:80-100` | 2.5–3 ms | Neutral | day |

- **T3.1** is the highest ceiling *and* the best tail win (each removed fork-join barrier removes a
  Windows park/unpark tail event). Riskiest: must prove rebuild reads *this* tick's positions to
  build *next* tick's grid (run after `integrate_motion`), with bit-identical output for 100 ticks.
- **T3.4 is GATED by T3.1** — if the schedule overlap hides L1 aggregation under steering,
  parallelising it becomes worthless. Decide T3.1 first, then re-evaluate.

---

## ❌ Ditched / stale — do NOT revisit (and why)

- **Flat 2D dense-array grid** — *already done.* The grid is already a flat dense `Vec` indexed
  `ly*width+lx` (`grid.rs:76,343`); there is no `FxHashMap`. The backlog item describes a shipped migration.
- **Parallelise grid-rebuild prefix-sum** — marginal; the prefix-sum already runs only over
  non-empty cells (~28K of 252K), and count/scatter are already parallel. Sub-ms for a new barrier.
- **`Changed<>`/`With<>` query filters (~25–30% claimed)** — double-counts the throttle. At 900K
  nearly every creature moves every tick, so `Changed<Position>` matches ~all of them; the real
  "skip unchanged" mechanism is the divisor-8 frequency throttle, already in place.
- **Movement hot/cold split / archetype consolidation** — movement is only 8.3 ms and already a
  tight fused loop; the cited "61%" is from another codebase. T3.2 (perception SoA) is the
  higher-value version of the same idea.
- **SIMD perception/distance math** — <1 ms for high effort; Rayon already parallelises across
  creatures, and 2D float pairs don't align cleanly to 4/8-wide SIMD. Revisit only after Tier 1.
- **`#[repr(C, align(16))]` on components** — `Perception` hot fields already fit a cache line; padding is waste.
- **Slot-map export / "kill `par_sort`"** — there is no full sort in the hot loop; the only sort
  is `select_nth_unstable_by` (partial, bounded by `MAX_PERCEIVED_NEIGHBORS = 7`).
- **Persistent thread-local scratch buffers** — *already done* (`perception/systems.rs:42-49`).

---

## The test protocol (the keep/ditch gate)

Every item runs this before it earns ✅ KEEP. Build once: `cargo build --release --features dev-tools`.

1. **Baseline** (once per experiment): `--pop 900000` and `--pop 200000`, each across **5 seeds**
   (11, 42, 99, 137, 2025). The across-seed p99 std at 900K is your **noise floor** — if it
   exceeds 2 ms, no single-seed run is evidence.
2. **200K diagnostic** (`--out`, `diff_reports`): confirm the change moves the *expected phase
   only*. Another phase moving >5% is a compensation signal — investigate.
3. **900K verdict** (5 seeds, `--out`): the real test, because density bites near 1M in ways 200K
   hides.
4. **Phase attribution**: target phase's p99 drops ≥ the total wall p99 drop; no other phase p99
   rises >1.5 ms (transfer budget).
5. **Ceiling**: `--find-max --low 800000 --high 1100000 --coarse-step 50000 --tolerance 10000`,
   3 seeds, baseline vs candidate.
6. **Sweep shape**: `--sweep 200000→1000000 step 100000` — candidate must not be worse than
   baseline at *any* population in [500K, 1M] (a high-density crossover = density-regressive → ditch).

**Pass bar (KEEP):** `Δp99_wall @900K ≤ −3 ms` **AND** `|Δp99| > 2 × noise_floor` **AND** no
per-phase p99 regression >2 ms on any seed **AND** ceiling +≥20K creatures **AND** no sweep
density-inversion. For Tier 2, **also** the trophic-stability canary (±20% population). Defer
(don't ditch) anything that lands between −1 and −3 ms.

---

## Recommended attack order

*(Revised after the Tier-1 findings above — micro-opts proved below the noise floor.)*

1. ✅ **Prereqs L1–L3** — done.
2. ✅ **T1.1 range trim** — kept for correctness; ❌ **T1.2/T1.3** ditched (below noise).
3. **T2.5 stochastic perception phase** — NEXT. Directly attacks the p99 tail (the actual
   bottleneck) by de-synchronising perception ticks; cheap (per-creature phase offset, no new
   state); biological (reaction-time jitter). Best ratio of tail-impact to effort.
4. **T3.1 schedule overlap** — removes fork-join barriers, the other tail driver. Bigger/riskier.
5. **T2.2 hunger-gating** — the largest raw cut (deletes perception ticks for fed creatures), but
   needs an energy/hunger state first — check whether one exists before committing.
6. **T2.1 range cap / T2.3 / T2.4 / T3.2 / T3.3** as the gap to 1M narrows.

---

**Document Owner:** Pillar 1 (Prove Scale) · **Created:** 2026-06-21 · update Status as items are tried.
