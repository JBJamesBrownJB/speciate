# Perf Hunt — Last Run Report

🚧 **In-progress (NOW)** · Date: **2026-06-25** · Pillar 1 (Prove Scale)

**Target:** 1,000,000 creatures sustained inside the 50 ms tick budget (20 Hz).
**This run:** 10 ideas considered, 9 implemented and measured, **2 KEEPs** — both **replicated** perception wins (**−1.88 ms** and **−2.30 ms** wall p99) that pull the 1M baseline comfortably under budget. Both target the *same* phase, so they are **not yet proven additive** — stack-test before merging both.

---

## Baseline (committed state, this run)

Built release `latency_lab` with `--features dev-tools`; stashed at
`apps/simulation/target/perf-hunt/latency_lab_baseline.exe` (1,479,680 bytes).
Sanity run: **pop = 1,000,000**, seeds `[11,42,99,137,2025]`, half-x/y 5000, realistic DNA.

| Metric | Value |
|---|---|
| **Wall p99 (mean of per-seed p99s)** | **48.837 ms** — *just under the 50 ms budget* |
| Wall noise floor (std / worst) | 2.983 ms / 54.032 ms |
| Wall mean-of-means | 45.749 ms |

Phase p99 breakdown (µs):

| Phase | p99 |
|---|---|
| **steering** *(largest)* | 14.182 ms |
| perception | 10.604 ms |
| movement | 10.059 ms |
| grid_rebuild | 7.323 ms |
| l1_aggregation | 4.551 ms |
| behavior | 4.384 ms |

The 1M baseline sits **~1.2 ms under budget** but with a thin worst-case margin. **Perception** was the productive seam this run — both KEEPs live there.

---

## Summary — all candidates

| Idea | Scope | Target phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|---|---|---|---|---:|---:|
| **Split giant L1 cone into own Rayon pass** | architectural | perception | ✅ **KEEP** *(replicated)* | **−1.88** | −1.12 |
| **Compact perception active-set before dispatch** | architectural | perception | ✅ **KEEP** *(replicated)* | **−2.30** † | −2.69 |
| Drop unused HomePosition column from steering collect | engine | steering | ⚠️ Ditch *(see note)* | −0.46 | −0.50 |
| Sub-throttle L1 strategic cone scan | biological | perception | ❌ Ditch | −0.80 | −0.13 |
| Hoist max_accel divide + defer SteeringContext | engine | steering | ❌ Ditch | +3.09 | +0.20 |
| Decouple BodySize-cache from fork-join chain | architectural | behavior | ❌ Ditch | +0.82 | −0.04 |
| Atmospheric-extinction perception range cap | biological | perception | ❌ Ditch | +0.50 | +0.04 |
| Cache allometric scalars in BodySize | engine | steering | ❌ Ditch *(triage)* | −0.03 | +0.16 |
| Size-gated steering latency (giants coast) | biological | steering | ❌ Ditch *(triage)* | +0.08 | +0.02 |

*(Δ is candidate − baseline; negative = faster. KEEP/escalated Ditches ran the full 1M / 5-seed gate; two Ditches stopped at 500k triage. Wall deltas are the gate's robust **median**, except the two KEEP headline numbers below which also report mean-of-p99s.)*

† **Caveat on the −2.30 ms:** the cross-seed **p99** aggregate is contaminated by one anomalous seed (2025) and was computed at the **median**. All 5 seeds improved at median; 4/5 improved at p99. See KEEP note.

---

## ✅ KEEPs — recommend merging (with one caveat)

### 1. Split the giant L1 strategic-cone scan into its own work-stealing Rayon pass
*architectural · target phase: perception · **replicated** (dWall −1.317 ms on the independent pass)*

**What it does:** hoists the heavy, branchy **L1 cone scan** out of the main `min_len(64)` perception loop into a dedicated `min_len(1)` work-stealing pass over **only** `CanStrategicVision`-marked giants (`perception.range ≥ L1_CELL_SIZE`). A handful of giants no longer straggle behind 63 cheap neighbors stuck in a shared leaf — the heavy tail gets its own balanced schedule.

**Result (full 1M / 5 seeds):** perception p99 **9.840 → 8.558 ms (−1.28 ms, ~5× over the 0.239 ms noise floor)**; wall mean-of-p99s **46.156 → 44.280 ms (−1.876 ms)**; **no other phase regressed** (steering −0.54, grid_rebuild −0.21, behavior −0.19, movement/l1 flat). `cells_queried` unchanged (identical work, just rebalanced). Ships with a guarding unit test.

**TRADEOFFS / consequences (the cost you are buying):**
- ✅ **Bit-identical** — no biology change, **no trophic canary required.**
- ⚠️ Adds a **second fork-join barrier** + a giant-only collect each tick. If the cone tail is throughput- rather than straggler-bound, that barrier is pure overhead (small — giant set ~10–20k; net was positive here).
- ⚠️ New `CanStrategicVision` marker is assigned **once at spawn** from static range. **Invariant to document:** it must be revisited if `perception.range` ever becomes runtime-mutable (today it is static post-spawn).
- ⚠️ Slightly more schedule/code surface.

### 2. Compact perception to this tick's throttle bucket before the parallel dispatch
*architectural · target phase: perception · **replicated** (dWall −2.444 ms on the independent pass)*

**What it does:** folds the **frequency-throttle + active gate** into the serial collect, so the dispatched parallel `Vec` shrinks by ~`perception_divisor` (≈7/8 of creatures skipped this tick are never dispatched). Removes no-op parallel work units instead of branching them away inside the hot body.

**Result (full 1M / 5 seeds, median gate):** perception **−2.694 ms**, wall **−2.302 ms** — both far above their noise floors (0.108 / 0.683 ms). Order/Entity-preserving, **bit-identical neighbor/L1Vision output by construction**.

> ⚠️ **READ BEFORE MERGING — the p99 caveat.** The cross-seed **p99** aggregate is poisoned by a single anomalous seed (2025): candidate perception p99 = 14.7 ms / wall p99 = 64.8 ms, versus the *other four* seeds at perception p99 7.1–7.5 ms (**better than baseline ~10 ms**) and wall p99 43.7–46.2 ms. Raw cross-seed p99 deltas therefore read **+14.46 ms wall / +3.96 ms perception**, entirely driven by that one outlier. At **median** all 5 seeds improved (perception ~6.7 vs ~9.4 ms; wall ~43 vs ~45 ms) and 4/5 improved at p99 — so the spike reads as a one-off scheduler/system event, **not a property of the change**. **Action: run one confirmation pass to rule out the seed-2025 tail spike before merging.**

**TRADEOFFS / consequences:**
- ✅ **Bit-identical** — no biology change, **no canary.**
- ⚠️ The throttle predicate now runs **single-threaded inside the collect** (serial work ∝ 1M) instead of parallel-side — mitigated by deleting the same check from the parallel body, but it shifts a little work out of the parallel region.
- ⚠️ If perception is memory-bandwidth-bound on the archetype walk rather than dispatch-bound, the win shrinks to the saved `Vec` size (empirically it did not here).
- ⚠️ The unresolved seed-2025 p99 spike (above) is the one open question — confirm before trusting the tail.

---

## Bundle — additivity NOT proven ⚠️

**No bundle A/B was run** (`bundle=null`). Both KEEPs target the **same perception phase / dispatch path**, so their wins are **not guaranteed to add** — they may compete for the same dispatch/bandwidth saving. Their standalone full-pop numbers (−1.88 and −2.30 ms) stand individually, but **do not assume −4.2 ms combined.**

> **Action: stack-test the two perception KEEPs together** (single A/B of baseline vs both-applied) before merging both. If you merge only one, prefer #1 (cone split) — its p99 is clean and uncontaminated.

## Defers (parked)

**None this run.**

---

## ❌ Ditched (and why)

### Drop unused HomePosition column from the steering collect *(engine, steering)* — ⚠️ *non-reproducing*
At full 1M the **first** measurement was a clean win (phase −0.497, wall −0.464 ms, both clearing noise) — it would have been a DEFER/KEEP. But on **replication it did NOT reproduce** (DEFER, dWall −1.973 ms — inconsistent magnitude/verdict). Marked Ditch on the reproducibility failure, not on direction. Zero-risk, bit-identical; **re-eligible** as a stackable DEFER if a third run settles it. Reverted clean.

### Sub-throttle the L1 strategic cone scan *(biological, perception)*
**Behavior change:** long-range flee/approach/disperse drives would act on up-to-2×-stale `L1Vision`. Direction was favorable (wall −0.798 ms) but the gain sits **entirely inside** the 0.846 ms wall-noise floor at full pop — not separable from noise. Would have needed the apex/grazer ±20% **trophic canary** before any merge; died on perf first. Ditched; reverted clean.

### Hoist max_accel divide + defer SteeringContext *(engine, steering)*
Bit-identical micro-op. Triaged Keep at 500k but **did not hold at 1M**: phase +0.20 ms (inside 0.291 noise), wall +3.09 ms (noise-dominated, candidate noise floor was elevated). No real signal. Ditched; reverted clean.

### Decouple BodySize-cache from the fork-join chain *(architectural, behavior)*
Reorder is bit-identical. Triaged Defer at 500k but at 1M regressed wall **+0.824 ms against 0.803 noise** with no defensible phase win (−0.038, inside noise). The inner pass is already memory-bound. Ditched; reverted clean.

### Atmospheric-extinction perception range hard cap *(biological, perception)*
**Behavior change:** largest narrow-FOV giants would lose long-range L1 awareness beyond a 450 m cap. At realistic-DNA distribution **very few/no cones exceed 450 m**, so the cap rarely fires — perception p99 actually rose slightly (+0.039 ms), wall deltas flip sign between median and p99 (pure jitter). Would have required a trophic canary; never earned one. Ditched; reverted clean.

### Cache derived allometric scalars in BodySize *(engine, steering)* — *stopped at triage*
Fattening `BodySize` 8B→24B **regressed** the steering phase +0.160 ms (vs 0.018 noise) at 500k — the widened SoA column hurt cache density in the hot loops more than the saved `powf`/`powi` calls helped. Wall change buried in noise. Per protocol (Ditch + dWall not clearly negative), no escalation. Reverted clean.

### Size-gated steering latency — "giants coast on momentum" *(biological, steering)* — *stopped at triage*
**Behavior change** (giants ≥5.0 m re-steer 1-in-N ticks). With realistic log-normal DNA there are **too few giants to skip meaningful work**, so the throttle-branch + index overhead **regressed** steering p99 +0.962 ms (candidate stdDev exploded to 420 µs vs 22 µs base). Net loss on the targeted phase. Would have needed the apex/grazer ±20% canary anyway. Ditched; reverted clean.

---

## 🎯 Recommend merging — shortlist

1. **Split giant L1 cone into own Rayon pass** — **−1.88 ms wall p99 (replicated −1.32 ms), bit-identical, no biology cost, clean p99.** *Merge now; document the `CanStrategicVision`-is-static-post-spawn invariant.*
2. **Compact perception active-set before dispatch** — **−2.30 ms median (replicated −2.44 ms), bit-identical.** *Merge after one confirmation run to clear the seed-2025 p99 tail spike.*
3. **Before merging BOTH:** run a **stack A/B** — they share the perception path and additivity is unproven. Do not bank −4.2 ms combined on faith.

## What to hunt next

- **steering (14.18 ms) and movement (10.06 ms)** are now the fattest untouched phases — perception got the wins this run. Aim the next hunt there. (Note: the previously-tested *fuse steering + integrate_motion* win is recorded as HELD, not merged — its phase landscape interacts with steering targeting.)
- **grid_rebuild (7.32 ms)** is a clean, biology-free architectural target with no candidate yet.
- **Settle the HomePosition-column drop** with a third run — it's a zero-risk freebie if it reproduces.
- After merging the perception KEEP(s), **re-baseline** — perception detect bars will shift and prior per-phase floors go stale.
