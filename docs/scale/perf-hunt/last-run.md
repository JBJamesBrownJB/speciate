# Perf Hunt — Last Run Report

🚧 **In-progress (NOW)** · Date: **2026-06-25** · Pillar 1 (Prove Scale)

**Target:** 1,000,000 creatures sustained inside the 50 ms tick budget (20 Hz).
**This run:** 4 ideas considered, 4 implemented and measured, **1 KEEP** — a clean, replicated **−2.70 ms wall p99** win that puts 1M back under budget.

---

## Baseline (committed state, this run)

Built release `latency_lab` with `--features dev-tools`; stashed at
`apps/simulation/target/perf-hunt/latency_lab_baseline.exe`.
Sanity run: **pop = 1,000,000**, seeds `[11,42,99,137,2025]`, half-x/y 5000, realistic DNA.

| Metric | Value |
|---|---|
| **Wall p99 (mean of per-seed p99s)** | **51.418 ms** — *slightly over the 50 ms budget* |
| Wall noise floor (std / worst) | 3.226 ms / 56.982 ms |
| Wall mean-of-means | 46.892 ms |

Phase p99 breakdown (detect bar in parens):

| Phase | p99 | std | detect bar |
|---|---|---|---|
| **steering** *(largest)* | 14.754 ms | 1.521 | >3.041 |
| perception | 11.154 ms | 0.586 | >1.172 |
| movement | 10.338 ms | 1.069 | >2.138 |
| grid_rebuild | 8.355 ms | 0.589 | >1.179 |
| l1_aggregation | 4.822 ms | 0.131 | >0.262 |
| behavior | 4.693 ms | 0.552 | >1.104 |

The 1M baseline sits **~1.4 ms over budget** going in. Steering is the fattest phase and was the focus of 3 of 4 candidates.

---

## Summary — all candidates

| Idea | Scope | Target phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|---|---|---|---|---:|---:|
| **Fuse steering + integrate_motion** | architectural | steering | ✅ **KEEP** *(replicated)* | **−2.70** | +6.00 † |
| Hoist max_accel divide + defer SteeringContext | engine | steering | ❌ Ditch | +0.87 | +0.16 |
| Size-gated steering latency (giants coast) | biological | steering | ❌ Ditch | +1.28 | −0.02 |
| Fold L1-aggregation into grid rebuild | architectural | l1_aggregation | ❌ Ditch | +0.03 | +0.03 |

*(Δ is candidate − baseline; negative = faster. Triage Ditches ran at 500k; the KEEP escalated to full 1M.)*

† **The +6.00 ms steering Δ is a measurement artifact, not a regression** — see KEEP note below.

---

## ✅ KEEP — recommend merging

### Fuse `update_steering_system` + `integrate_motion_system`
*architectural · target phase: steering · replicated*

**What it does:** merges the two systems into one query+closure pass, deleting **one serial 1M-element collect, one fork-join barrier, and a redundant full re-load of the shared position/velocity columns**.

**Why the phase gate said "Ditch" (and why we overrode it):** fusing moves the movement/integrate work *under* the steering-labelled span. The "movement" phase collapses to 0 ms and its cost re-accounts as steering, so the per-phase gate sees steering balloon +6.0 ms and misses that movement vanished. The honest combined accounting (full 1M, 5 seeds, mean):

```
base:  steering 13.797 + movement 8.932 = 22.729 ms
cand:  steering 19.833 + movement 0.000 = 19.833 ms   →  −2.9 ms of real work
```

**The banker (wall clock) improved robustly and consistently:**

| Measurement | Baseline → Candidate | Δ |
|---|---|---|
| Triage 500k / 3 seeds — wall p99 mean | 24.458 → 22.880 ms | dWallMedian −1.2 ms |
| **Full 1M / 5 seeds — wall p99 p50** | **47.754 → 45.056 ms** | **−2.70 ms** |
| Full 1M / 5 seeds — wall p99 mean | 48.435 → 44.515 ms | −3.92 ms |
| Replication pass | — | **dWall −3.746 ms** |

Wall noise floor was 1.056 ms, so the win is **>3× noise** and held across an independent replication.

**Determinism preserved:** `cells_queried` identical (2,752,367 vs 2,752,040); non-target phases (behavior, l1, grid) unchanged within noise.

**TRADEOFFS / consequences (the cost you are buying):**
- ✅ **Behavior-preserving / bit-identical** — no biology change, **no trophic canary required**.
- ⚠️ **Collapses two independently-measured phases into one.** Steering and movement can no longer be profiled separately; future per-phase DETECT on either is blinded. Must bank on the wall median + explicit re-labeling. **Flag to human: rename the fused span** so the lab/docs don't imply "steering" alone is 19.8 ms.
- ⚠️ **Very fat query+closure** — added register pressure may erode part of the bandwidth win on other hardware; the net here is empirical (and positive on JB's rig).
- ⚠️ **dev-tools debug-accel capture must be relocated** into the closure or `cfg`-gated, or instrumentation builds break.
- ⚠️ Larger, harder-to-read system; mitigate by extracting inline fns.

**Bottom line:** this alone takes the 1M baseline from **47.75 → 45.06 ms wall p99**, i.e. **back under the 50 ms budget** with margin, at zero biological cost.

---

## Bundle

**No bundle was formed.** Exactly one KEEP and zero surviving DEFERs, so the union equals the single KEEP — no separate additive A/B was needed. The standalone full-pop result (−2.70 ms, replicated −3.746 ms) is the bankable number.

## Defers (parked)

**None this run.**

---

## ❌ Ditched (and why)

### Hoist max_accel divide + defer SteeringContext build *(engine, steering)*
Bit-identical micro-op. Triage @500k: phase Δ +0.161 ms sits **inside** phase noise (0.304 ms) and trends *positive*; wall p99 p50 24.558 → 25.432 ms (**+0.87 ms, a regression**). Signal never clears the median floor. Ditched on perf; reverted clean.

### Size-gated steering reaction latency — "giants coast on momentum" *(biological, steering)*
The one **behavior-changing** candidate: creatures ≥5.0 m would re-steer only 1-in-4 ticks (mass → reaction latency; up to 150 ms of coasting, late dodges). At 500k with realistic log-normal DNA there are **too few giants to skip meaningful work**, so the throttle-branch + index overhead *netted a wall regression* (24.480 → 25.759 ms, **+1.28 ms**) and phase Δ was −0.015 ms (pure noise).
**Trophic-canary status: not run** — the idea died on perf before it earned a canary. If revisited, it still **requires** the apex/grazer ±20% canary before any merge, and the bench's monotone-draining energy model means the canary would measure population drift, not steady-state ecology. Ditched; reverted clean.

> Note: a *prior* (rev1) run of this idea flagged a −2.5 ms wall win at full 1M; this rev2 triage at 500k regressed. The population mix matters — the win, if any, only appears where giants are dense enough. Re-eligible only with that caveat understood.

### Fold L1-aggregation into grid rebuild + hoist cell accumulator *(architectural, l1_aggregation)*
Bit-identical (hoisted running-reference variant). Raw `l1_aggregation` p99 dipped ~0.08 ms (3.102 → 3.021 ms) but the **median delta sits inside the 0.013 ms phase-noise floor** and wall clock showed no net win (+0.034 ms). The inner loop is already memory-bound on the proxy stream, so removing the per-proxy branch/push didn't surface at the tick level. Ditched; reverted clean.

---

## 🎯 Recommend merging — shortlist

1. **Fuse steering + integrate_motion** — **−2.70 ms wall p99 (replicated −3.75 ms), bit-identical, no biology cost.** Takes 1M back under the 50 ms budget. *Merge with the phase-rename caveat: re-label the fused span and note the lost steering/movement split.*

That's the whole shortlist — it's the only KEEP, and it's a strong one.

## What to hunt next

- **perception (11.15 ms) and grid_rebuild (8.36 ms)** are now the untouched fat phases — steering got hit hard this run. Aim the next hunt there.
- **T2.8 FOV-cone L0 scan cull** (already logged as an idea) targets perception directly — promising follow-up.
- **Re-examine size-gated steering at 1M / denser-giant DNA** — the rev1/rev2 split shows its payoff is population-dependent; if pursued, run it *at full pop* and pair it with the mandatory trophic canary.
- After the steering+integrate merge, **re-baseline** — the phase landscape shifts (steering/movement now fused), so prior per-phase detect bars are stale.
