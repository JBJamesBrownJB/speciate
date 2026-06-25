# Perf Hunt — Run Report (2026-06-26)

🚧 In-progress NOW · Pillar 1 (Prove Scale)

**Config:** 10 ideas, all 10 implemented & measured. Full pop **1,000,000** · triage pop **500,000** · seeds `11,42,99,137,2025` · realistic DNA. Decision metric is **wall p99** through the phase-aware latency lab (baseline-first back-to-back A/B, serial).

## Baseline (the wall we are pushing on)

Wall **p99 = 49.97 ms** against a **50 ms** tick budget — the live baseline is *already at the ceiling*. Any regression overruns the tick, so this run's bar is high: only clear, above-noise wall wins count, and a "phase got faster but wall didn't" result is a Ditch.

Phase p99 breakdown (µs): **steering 14,971** (~30% of wall, the fat target) · movement 10,512 · perception 8,445 · grid_rebuild 8,410 · behavior 4,692 · l1_aggregation 4,647.

## Scoreboard

| Idea | Scope | Target phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|---|---|---|---|---:|---:|
| Native Bevy `par_iter_mut` — kill the 1M collect | architectural | steering | **KEEP** | **−7.258** | −4.230 |
| Fuse behavior_transition into steering | architectural | steering | DITCH* | −2.576 | +1.284 |
| Lower steering Rayon `min_len` (256→64) | architectural | steering | DITCH† | −2.218 | −0.492 |
| Avoidance reaction-latency jitter (every-other-tick) | biological | steering | DITCH | −2.107 | +0.171 |
| Branchless octant — kill perception atan2 | engine | perception | DITCH† | −1.479 | −0.761 |
| BodySize powf→sqrt allometry | engine | steering | DITCH | −0.451 | −0.092 |
| Hand-merged movement turn-limiter (powf+atan2-free) | engine | movement | DITCH | +0.321 | −0.199 |
| max_accel from cached inv_sqrt_length | engine | steering | DITCH | +0.301 | +0.039 |
| Cap tracked neighbors 7→5 | biological | steering | DITCH | +0.064 | −0.045 |
| Skip no-op accel cap for wanderers/catatonic | engine | steering | DITCH | +1.011 | +0.574 |

Δwall is the wall-clock delta the verdict tool reported (median at triage-only rows, p99-class at escalated rows). Negative = faster. `*` and `†` explained below.

---

## KEEP — merge candidate

### Native Bevy `par_iter_mut` for steering + movement — deletes the serial 1M collect
**Δwall p99 −7.258 ms** (replicated −7.345 ms) · Δsteering −4.23 ms · phase noise 0.12 ms · wall noise 0.39 ms · **architectural · bit-identical**

The biggest win of the run by a wide margin (>18× the wall noise floor). The old hot loop did `iter_mut().collect()` into a 1M-entity `Vec` every tick, then `par_iter_mut()` over that gather — once for steering, once for movement. Switching to native Bevy `par_iter_mut` (enabling the `multi_threaded` feature + `ComputeTaskPool` init, with the `SingleThreaded` *system* executor preserved for determinism) deletes the per-tick allocation in **both** phases. That is why the wall win (−7.26 ms) is larger than the steering-phase win (−4.23 ms): movement shed its gather too. At 1M: steering p99 13,828→9,702 µs, movement 9,361→7,183 µs, wall p99 44,268→37,406 µs.

**Tradeoff / cost the human is buying:**
- **Behavior-preserving** — no gameplay change. Only a **determinism canary** is required before merge: `cells_queried` identical across all 5 seeds. *Not* a trophic canary (intent unchanged).
- **Catastrophic-failure mode if mis-wired:** if `ComputeTaskPool` is not initialized with >1 thread, the phases silently run *serial* and you get a massive regression instead of a win. The merge MUST assert thread count > 1.
- **Confidence caveat:** the earlier RCA (#47) attributed the wall cost to the fork-join **barrier**, not the collect — Bevy's batching barrier could in theory be heavier than the hand-tuned `with_min_len(256)` Rayon one. The measured result refutes that worry here, but it is the thing to watch if the win fails to reproduce on another machine.

---

## BUNDLE — none assembled

Only one KEEP this run, so there was nothing to stack and no additivity check to run.

**Important non-additivity note for the human:** the two strongest architectural results — the KEEP (`native par_iter`) and the DITCH* (`fuse behavior_transition`) — **attack the same resource**: the serial 1M collect + the fork-join barrier around steering. They are **mutually exclusive, not additive**. Pick one. The native-par-iter KEEP already delivers the larger wall win and is the recommended path; the fuse is the fallback if the par_iter thread-pool wiring proves fragile on a target machine.

---

## DEFERS — none

No candidate landed in the "real phase win but sub-noise on wall, park for later" bucket this run. Every non-keep either regressed wall or washed out entirely.

---

## DITCHED — and why (the instructive failures)

### Fuse behavior_transition into steering — DITCH* (mechanically) but a real wall win
**Δwall p99 −2.576 ms** (>20× wall noise) · Δsteering **+1.284 ms** · architectural · bit-identical

**Read this one carefully.** The phase-targeted verdict tool returned DITCH because this is a *fusion*: the standalone `behavior_transition_system` is folded into the top of `update_steering_system`, so the behavior phase collapses to 0 and its ~4.7 ms reappears *inside* steering. By the steering-phase metric alone that looks like a +1.28 ms regression — hence the mechanical Ditch. **But the wall clock is the real signal and it is a clean −2.576 ms win** (removing one serial collect + one fork-join barrier; combined steering+behavior p99 18,285→14,933 µs). This is effectively a KEEP that the phase-scoped tooling can't score correctly. It loses the standalone behavior-phase timer (weakening per-phase detection on the leanest phase) and fattens the steering tuple with a `Brain` mut column (cache/register pressure — see ledger #36). **Mutually exclusive with the native-par-iter KEEP** (same barrier). Recommend: keep par_iter; hold this as the documented fallback.

### Lower steering Rayon `min_len` 256→64 — DITCH† (not reproduced)
**First run: Δwall p99 −2.218 ms (KEEP at 1M).** On replication it did **not reproduce** (dWall +0.657 ms). Bit-identical, zero behavior risk, but the signal is unstable — the finer batches' work-stealing win on the heavy-tailed avoidance loop is real on some runs and washed by per-batch scheduler overhead on others. Not safe to merge on one good run. Cheap to revisit *after* the par_iter KEEP lands, since par_iter changes the batching substrate entirely.

### Avoidance reaction-latency jitter — DITCH (biological)
**Δwall p99 −2.107 ms was a baseline-noise artifact.** One spiking baseline seed (grid_rebuild noise 1,081 µs) inflated the baseline p99; the robust *median* wall delta was −0.020 ms (flat) and the targeted steering phase actually *regressed* +0.171 ms. No real win. Tradeoff had it merged: **behavior change** — dodging lags up to one 50 ms tick, so fast predator-prey intercepts feel sluggish; would have required a **trophic canary** (apex & grazer within ±20%). Ditched on perf grounds before that cost was ever worth paying.

### Branchless octant (kill perception atan2) — DITCH† (not reproduced)
**First run −1.479 ms wall (KEEP, escalated).** On replication it dropped to −0.317 ms (DEFER-class). The one remaining transcendental in the perception hot loop; bit-identical iff the branchless wedge boundaries match `atan2+rem_euclid` at every angle (guarded by an exhaustive 0.5° sweep test). Genuinely promising but the win is smaller and less stable than first measured — re-measure on a quiet machine before trusting it.

### The micro-op steering tweaks — all DITCH, all instructive
`max_accel from cached inv_sqrt` (+0.301 ms), `BodySize powf→sqrt` (−0.451 ms within noise), `skip no-op accel cap for wanderers` (+1.011 ms, a real regression), `cap neighbors 7→5` (+0.064 ms). **Common lesson:** steering at this scale is **not ALU-bound** — it is dominated by neighbor-cache traversal in the avoidance branch. Strength-reducing the arithmetic (`powf`→`sqrt`, divides→cached reciprocals) gets folded by `-O3` anyway and registers nothing above noise; *adding* per-entity branches to skip work (the wanderer/catatonic gate) actively *costs* more than the work it skips, because the elided cap path was already cheap and branch-predicted. Confirms ledger #15/#19/#50: per-creature branch additions to this loop wash or regress.

### Hand-merged movement turn-limiter — DITCH
Δmovement −0.199 ms (a real but tiny phase win) but Δwall **+0.321 ms** — the movement phase got marginally faster while wall did not improve. The powf(1.0)→divide half is bit-identical; the atan2-free half is not (±1 ULP on clamped headings, would need a determinism canary). No wall payoff to justify the risk.

### Cap tracked neighbors 7→5 — DITCH (biological)
No measurable win (Δwall +0.064 ms, deep in noise). Would have been a **behavior change** (coarser dense-pack avoidance/schooling, smallest grazers lose an avoidance margin) requiring a trophic canary — all cost, no perf upside. Ditched.

---

## Recommend merging

1. **Native Bevy `par_iter_mut` (kill the 1M collect)** — **−7.26 ms wall p99, replicated.** Bit-identical, behavior-preserving. The single highest-value, lowest-behavioral-risk change this run. *Gate:* assert `ComputeTaskPool` thread count > 1, and run the `cells_queried` determinism canary across all 5 seeds before merge. This alone moves the live baseline from 49.97 ms toward ~42.7 ms — out of overrun territory.

That is the only merge-now item. **Do not also merge the fuse** — it fights the same barrier and is mutually exclusive.

## What to hunt next

- **Re-measure the two "not reproduced" candidates on the post-par_iter tree:** `min_len 256→64` and `branchless octant`. par_iter changes the batching substrate, so their economics shift — both are bit-identical and cheap to retry, and the octant kill is the last transcendental in perception.
- **Stop micro-optimizing steering arithmetic.** This run proved it is memory/neighbor-cache-bound, not ALU-bound. The next steering win has to come from the **access pattern** (cache layout of the neighbor cache, SoA-ifying the avoidance read set) or from **doing less work biologically** (Golden-Zone skips that survive a trophic canary), not from shaving FLOPs.
- **Re-profile after the KEEP lands.** Deleting ~7 ms reshapes the phase ranking; perception (8.4 ms) and grid_rebuild (8.4 ms) likely become the new joint-second targets behind a slimmer steering.
