# 🚧 Perf Hunt — Last Run (2026-06-25)

**Verdict in one line: nothing to merge.** Five candidates hunted, five ditched. No regressions land in the tree — the baseline is untouched and the hunt was clean (back-to-back A/B, baseline-first, clean-tree guard passed before and after every candidate).

## Baseline (1M pop, 5 seeds: 11,42,99,137,2025)

| Metric | Value |
|---|---|
| Wall p99 (mean of per-seed p99s) | **49.234 ms** (std 0.683 ms, worst 50.021 ms) |
| Wall mean-of-means | 46.318 ms |
| Headroom under 50 ms tick budget | **~0.77 ms (~1.5%)** |

Per-phase p99 (ms): **steering 14.0** · **perception 11.0** · movement 9.3 · grid_rebuild 8.1 · l1_aggregation 4.8 · behavior 4.3. Steering is the single largest phase; perception second. Those are the two fattest targets and where four of five ideas aimed.

## Summary table

| Idea | Scope | Target phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|---|---|---|---|---:|---:|
| Steering avoidance: sqrt-free closing-direction pre-filter | engine | steering | DITCH | +0.150 | −0.113 |
| T3.1 software-pipeline grid rebuild overlapping prev-tick steering/movement | architectural | grid_rebuild | DITCH | +4.613 | +1.947 |
| T2.2 sated-predator skip of L1 strategic cone scan (Golden Zone) | biological | perception | DITCH | +0.592 | −0.040 |
| T2.5 stochastic per-entity perception phase offset | engine | perception | DITCH | +0.240 | +0.078 |
| Steering wander: seeded PRNG + skip RNG for exhausted creatures | biological | steering | DITCH | +1.526 | +0.222 |

> Δ convention: negative = faster (improvement). Every Δwall p99 above is **positive** — none of the five made wall p99 better.

## ✅ KEEPS

**None.** No candidate produced a bankable wall-p99 win.

## 📦 BUNDLE

**None formed.** A bundle requires surviving KEEPs/DEFERs to stack into a union A/B; this run had zero of each, so there was nothing to combine.

## 📋 DEFERS (parked)

**None survived.** One candidate (T2.2) earned a tentative DEFER at the 500k triage but **failed on escalation to 1M** (see below), so it is not parked — it is ditched.

## ❌ DITCHED (with why)

### 1. T2.2 — Sated-predator skip of the L1 cone scan (biological) — the near-miss
The only idea that escalated. At 500k it looked real: perception p99 −0.252 ms at ~4.7σ over noise, wall p99 −0.241 ms (negative, sub-noise) → DEFER → escalated to 1M.
**At 1M it collapsed:** perception gain fell to −0.040 ms vs 0.249 ms phase noise (gone), and wall p99 drifted to **+0.592 ms** (slightly worse, sub-noise). The win did not survive scale.
**Why it died — and the honest tradeoff that predicted it:** the gate only pays off when a meaningful fraction of predators are *sated*. But there is **no feeding/energy-restore loop in production** (`restore_energy` exists only in tests), so under the bench drain energy is monotone-draining and the sated cohort is thin and shrinking. The instantaneous saving was too small to clear the p99 tail at full pop.
*Behavior cost it would have carried if merged:* sated predators lose long-range strategic awareness and become pulse-hunters (start pursuit only below `LOW_ENERGY_THRESHOLD`), not continuous trackers. It would have required a trophic canary (apex + grazer counts within ±20% over a ≥5000-tick, 200k mixed-DNA run). Moot now — ditched on perf alone.

### 2. T3.1 — Software-pipeline grid rebuild overlapping steering/movement (architectural) — the worst regression
Hard regression at the 500k triage: grid_rebuild p99 **+1.947 ms** (vs 0.063 ms noise) and wall p99 **+4.613 ms** (vs 0.192 ms noise). The `rayon::join` overlap plus the `collect_grid_snapshot` Vec allocation and L1 double-buffer aggregation cost *more* than the serial schedule it replaced — the movement and grid arms contend for the shared Rayon pool rather than overlapping cleanly at this scale. Did not escalate. (This echoes the prior-run lesson that parallel L1 aggregation regressed on real hardware.)

### 3. Steering wander: seeded PRNG + skip RNG for exhausted creatures (biological)
Slower on *both* the target and wall: steering p99 **+0.222 ms** (noise 0.083) and wall p99 **+1.526 ms** — an order of magnitude over the 0.170 ms wall noise. The xorshift jitter + per-creature `is_exhausted()` branch + `PhysicsTick` resource read did not beat `thread_rng` and perturbed the hot `par_iter` loop. Clear regression; ditched at triage.

### 4. Steering avoidance: sqrt-free closing-direction pre-filter (engine)
A wash. Steering p99 −0.113 ms sits *inside* the 0.137 ms phase noise (not a real signal), and wall p99 moved the wrong way (+0.150 ms). The pre-filter saves a sqrt+2 divides only on already-rejected receding neighbors, but adds a divide on the converging path — and in dense 500k packs most in-range neighbors are converging, so the hot path got marginally *more* work. Bit-identical, but no win. Ditched at triage.

### 5. T2.5 — Stochastic per-entity perception phase offset (engine)
Slight regression: perception p99 **+0.078 ms** (noise 0.024) and wall p99 **+0.240 ms** (noise 0.226). The gate is frequency-invariant — every creature still perceives at the same 1-in-N cadence — so it *cannot* reduce mean perception cost; it only reshuffles which absolute tick a giant lands on. On this hardware the reshuffle did not flatten the p99 tail and added a hash+add per gated entity. Ditched at triage.

## 🎯 Recommend merging

**Nothing.** Zero KEEPs, zero surviving DEFERs, no bundle. Merging any of these five would regress the tree. Leave the baseline as-is.

## 💡 What to hunt next

The baseline is **~0.77 ms (~1.5%) from the 50 ms budget at 1M** — close enough that a single ~1 ms phase win clears the headline. Where to aim:

- **Steering (14.0 ms, the fattest phase) — but stop nibbling the math.** Three steering micro-ops have now ditched (this run's pre-filter + wander-RNG; last run's hoist/defer-ctx). The phase-level math optimizations are below the detection bar at 1M. Next steering attempt should be **structural** (data layout / SoA access pattern / cache residency of the neighbor scan), not arithmetic shaving.
- **Perception (11.0 ms) — frequency-invariant reshuffles are a dead end.** T2.5 confirms that anything preserving total work/tick can't cut mean cost. To win on perception, *reduce work*: a real cull (Golden Zone skip) that survives 1M, not a tail-reshape. T2.2 was the right shape but starved by the missing feeding loop.
- **Unblock the biological Golden Zone gates by fixing the bench, not the gate.** T2.2 and the wander-skip both died on the same root cause: **no energy-restore loop exists**, so monotone drain makes the sated/non-exhausted cohorts unrepresentative under the bench. Until the lab models a steady-state energy distribution (or a feeding loop ships), every hunger/sated/exhaustion gate will under-measure. Consider seeding a realistic steady-state energy distribution into `latency_lab` so these gates can be evaluated honestly.
- **Architectural overlap needs a contention model.** T3.1's failure (shared Rayon pool, sub-linear overlap) suggests pipeline-overlap ideas won't pay until arms run on disjoint thread sub-pools or the snapshot-collect cost is eliminated. Park architectural overlap until that's designed.

---
*Method note: all five ran clean back-to-back A/B, baseline-first, on a verified clean tree; every patch applied/reverted cleanly and the clean-tree guard re-passed after each. Triage at 500k (seeds 11,42,99); escalation to 1M (seeds 11,42,99,137,2025) only on a clearly-negative wall signal. Ledger: `docs/scale/perf-hunt/ledger.jsonl`.*
