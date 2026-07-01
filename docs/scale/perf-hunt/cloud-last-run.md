# Cloud Perf-Triage — Last Run (2026-07-01)

> 🚧 **CLOUD TRIAGE SIGNALS ONLY — NOT AUTHORITATIVE.** These numbers come from a **shared cloud VM** at **micro-pop = 10,000** (seeds 11, 42, 99), not the home rig at 1M. Per-phase timers on this VM are flaky (multiple candidates lost all per-phase instrumentation to a stale/contaminated shared `target-dir`, producing *spurious* negative Δphase artifacts — see per-row notes). Wall-clock and growth-exponent are the only signals trusted here. **This run NEVER merges an engine change and NEVER banks a win.** It only prioritizes candidates for a full home-rig `/perf-hunt`.
>
> ✅ **Hand-off is automatic:** the one PRIME candidate below was appended to `ledger.jsonl` as `verdict=CANDIDATE` with a `retest` field. The full `/perf-hunt` surfaces any ledger row carrying `retest` as a **PRIORITY re-test** and prefers it over fresh ideas — no changes to the home workflow required.

## Run config

- **count:** 5 candidates measured
- **micro-pop:** 10,000 · **seeds:** 11, 42, 99 · **growth sweep:** 1000..10000 step 2250
- **baseline:** wall p99 4.082 ms · growth exponent b = 0.361
- **fattest phases:** grid_rebuild, steering, perception, movement, l1_aggregation, behavior
- **Known environment caveat:** `cargo bench --bench simulation_bench` does **not compile** — a PRE-EXISTING, patch-unrelated bug (`benches/simulation_bench.rs:199` passes a 6-tuple to `rebuild_parallel`, which expects a 7-tuple). So `bench Δ%` is **0 / UNAVAILABLE** for every candidate this run.

## Results

Δ values in ms; **negative = faster**. `growth b` is the fitted sweep exponent (lower = scales better). Primes listed first.

| candidate | scope | target phase | phase_verdict | Δphase (ms) | Δwall (ms) | bench Δ% | growth b (base→cand) | proj 1M wall savings (ms) | PRIME? |
|---|---|---|---|---|---|---|---|---|---|
| cache-cell-index-scatter | engine | grid_rebuild | **Keep** | -0.053 | -0.077 | 0 (n/a) | 0.361→0.331 | **+3.30** | ✅ **YES** |
| grid-rebuild-thread-local-histogram-buckets | architectural | grid_rebuild | Ditch | -0.648† | -0.078 | 0 (n/a) | 0.361→0.559 | -31‡ | no |
| l1-aggregate-per-cell-reduce-not-per-proxy | engine | l1_aggregation | Ditch | +0.004 | +0.074 | 0 (n/a) | 0.361→0.433 | -9.0‡ | no |
| grid-prefix-sum-scratch-counts-no-atomic-loads | engine | grid_rebuild | Ditch | -0.706† | +0.164 | 0 (n/a) | 0.361→0.345 | +0.73‡ (noise) | no |
| l1-lod-skip-aggregation-of-tiny-only-cells | biological | l1_aggregation | Ditch | -0.255† | +0.474 | 0 (n/a) | 0.361→0.425 | -10.7‡ | no |

**proj 1M wall savings** = growth-aware extrapolation `wall·100^b` per side, then baseline − candidate (positive = candidate faster at 1M). Sign/magnitude are the signal, not the digits — `100^b` is very sensitive to a noisy `b`. ‡ These four are **approximate**: the run predated the `wall_base_ms`/`wall_cand_ms` capture, so absolutes were reconstructed from the global baseline (4.082 ms); every future run measures them directly.

† **Δphase is a BROKEN-COUNTER ARTIFACT, not a win.** In these candidate builds every per-phase timer reported 0.0 (contaminated shared `target-dir` after an interleaved `cargo bench` compile), so the "negative" Δphase is just baseline's real median differenced against a spurious 0. The trustworthy wall-clock and growth signals all point the **wrong** way for these three — see the ledger / RESULTS notes.

## Prime shortlist (→ home rig)

### 1. `cache-cell-index-scatter` — engine · grid_rebuild

**Why prime:** phase_verdict = **Keep**; grid_rebuild p99 1020µs → 864µs with **Δphase −0.053 ms clearly beyond the 4µs phase-noise floor**; wall p99 3126µs → 2871µs (Δwall −0.077 ms, wallNoise 23µs); and the **growth exponent improves 0.361 → 0.331** — i.e. it scales *better*, not just faster at 10k. Mechanism: caches the flat cell index computed in the counting-sort *count* pass and reuses it in *scatter*, dropping the redundant floor/index recompute (compute the key once, not twice). Protocol clean: clean-tree guard passed, applied + built (release + dev-tools `latency_lab`), reverted cleanly (`git diff --quiet` passed post-revert).

**What the home rig should confirm at 1M:**
- Replicate the grid_rebuild Δphase win at full population with real per-phase perf counters (the cloud VM's instrumentation was flaky — a forced clean rebuild was needed to get real numbers here).
- Confirm the growth-class improvement (b ↓) holds up the 1M ramp rather than washing out in home-rig noise.
- Restore a working criterion bench (fix the pre-existing `simulation_bench.rs:199` 6→7 tuple arity bug) so there's a real function-level `bench Δ%` signal this run couldn't produce.
- Watch `worstPhaseP99Regression` (27µs at 10k) stays negligible at scale.

**Artifacts:** patch stashed at `docs/scale/perf-hunt/candidates/cache-cell-index-scatter.diff`; ledger row appended to `docs/scale/perf-hunt/ledger.jsonl` (`verdict=CANDIDATE`, `origin=cloud-triage`, `retest` set).

## Not promoted (4)

`grid-rebuild-thread-local-histogram-buckets`, `l1-aggregate-per-cell-reduce-not-per-proxy`, `grid-prefix-sum-scratch-counts-no-atomic-loads`, `l1-lod-skip-aggregation-of-tiny-only-cells` — all **Ditch**. Every trustworthy metric (wall latency and/or growth exponent) shows flat-or-regression; the only "improvements" were zeroed-instrumentation artifacts.

**Logged as `CLOUD_TRIED` soft-exclusion rows** in `ledger.jsonl` (`verdict=CLOUD_TRIED`, `origin=cloud-triage`, **no `retest`**). This records that each was implemented + measured and came up short at cloud scale, so future *cloud* hunts don't re-propose and re-measure them — but it is deliberately **not** a permanent kill: with no `retest` and no `DO_NOT_REVISIT`, the home-rig `/perf-hunt` neither prioritizes nor buries them, because a noisy ≤10k shared-VM miss must not permanently kill an idea whose payoff might only appear at 1M. For the three † rows the contaminated per-phase counter reading was dropped (`dphase_ms=0`, artifact value kept in the note) so it can't later be misread as a phase win.

> ℹ️ This run executed on the pre-fix workflow (primes-only logging); the four `CLOUD_TRIED` rows were back-filled from this table BY HAND, and their `dwall_p99_ms` values are **median** deltas (the p99 absolutes are in each row's notes). Every subsequent run logs both kinds automatically through the guarded CLI (`node .claude/workflows/lib/ledger-cli.mjs append|lint`, which executes the tested builders in `cloud-ledger.mjs`) and computes `dwall_p99_ms` from the back-to-back A/B p99 absolutes — see `.claude/workflows/perf-hunt-cloud.mjs` (Log phase + the NORMALIZE-INLINE block).
