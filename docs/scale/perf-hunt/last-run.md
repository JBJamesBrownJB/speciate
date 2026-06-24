# 🚧 Perf Hunt — Last Run (2026-06-24)

**Population:** 1,000,000 (full) / 500,000 (triage) · **Seeds:** 11, 42, 99, 137, 2025 · **Ideas hunted:** 5 · **Kept:** 1

> **Baseline (1M):** wall p99 = **46.97 ms** of the 50 ms / 20 Hz budget (noise floor σ = 0.91 ms). **Steering is the dominant phase** (p99 13.56 ms), then perception (10.08 ms) and movement (9.45 ms).
> **The single KEEP this run buys back ~2.95 ms of wall p99 — roughly 6% of budget, ~0.6 ms inside the 50 ms ceiling becomes ~3.5 ms of headroom.**

---

## Headline

| Idea | Scope | Target phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|---|---|---|---|---:|---:|
| Parallelize L1 aggregation (Rayon owned-partition scatter) | architectural | l1_aggregation | ✅ **KEEP** | **−2.95** | **−1.52** |
| Bound L1 cone by inscribed circle + facing half-plane | engine | perception | ❌ DITCH | −1.08 | **+0.25** ⚠ |
| Hoist max_accel divide + defer SteeringContext build | engine | steering | ❌ DITCH | −1.16 | −0.34 |
| Cost-decorrelated perception throttle bucketing | architectural | perception | ❌ DITCH | −0.08 | **+0.48** ⚠ |
| Overlap update_body_size_cache with perception (break barrier) | architectural | behavior | ❌ DITCH | −0.78 | −0.22 |

Noise floors: wall p99 σ ≈ 0.91 ms (1M) / 1.27 ms (500k); per-phase σ ≈ 0.09–0.34 ms. A Δ only counts when it clears its own noise floor. ⚠ = the idea made its *own target phase slower*.

---

## ✅ KEEP — merge candidate

### Parallelize L1 aggregation — Rayon owned-partition scatter
**Δwall p99 −2.95 ms · Δl1_aggregation p99 −1.52 ms · no phase regressed (worst +(-)0.08 ms).**

Both the phase win (−1.52 ms vs 0.10 ms noise) and the wall win (−2.95 ms vs 0.91 ms noise) clear their noise floors by a wide margin — this is the only candidate that produced a credible, end-to-end wall-clock gain at 1M. Triage at 500k was a "Defer" (wall delta clearly negative), which is exactly why it was escalated to the full run, where it confirmed as a Keep.

**What it does:** replaces the serial non-empty-cell reduce that builds the L1 coarse grid with a Rayon owned-partition (row-band) scatter — each thread owns a disjoint band of L1 rows and writes only its own cells, so there is no contention and no float-add reordering.

**Tradeoff / consequence — the cost the human is buying:**
- **No behavior change, and provably so.** Output is identical data; owned-partition preserves float-add order, so it is bit-identical. Ships a TDD guard (`parallel_aggregation_byte_identical_to_serial`) and the gate's 100-tick replay holds. **No biological change, no trophic canary needed** — this is a pure mechanical re-parallelization, not a perception/behavior change.
- **Memory:** wants a fresh L1 buffer per tick unless double-buffered. Mitigation: keep two L1 buffers and ping-pong.
- **Re-read cost:** each thread re-reads the L0 non-empty list (cache-resident, cheap).
- **Latent soundness invariant:** correctness depends on L0 being immutable during aggregation. If a *future* change ever lets L0 mutate mid-aggregation, the snapshot soundness breaks. **Recommend: encapsulate/assert that invariant when merging** so it can't silently rot.
- **Floor behavior:** if Rayon fork overhead ever exceeds the ~3.6 ms phase at low thread counts, the lab gate ditches it — it can never become a regression.

---

## 🎁 Bundle — none this run

No bundle was formed. A bundle is a stacked union of DEFER-tier wins A/B'd together; this run produced **one KEEP and zero DEFERs**, so there was nothing to stack or test for additivity.

---

## 📋 Defers (parked) — none

No candidate landed in the DEFER tier this run. The four non-keeps were all clear DITCHes (see below), not parked-for-stacking.

---

## ❌ Ditched — and why

- **Bound L1 cone by inscribed circle + facing half-plane** *(perception)* — **made its own target phase slower (+0.25 ms vs 0.09 ms noise).** The extra per-column `sqrt` + bound computation cost more than the ~21% corner-cell savings buys back at this scale. Wall delta (−1.08 ms) sat inside the 1.27 ms triage noise floor — no real win. Triage-only; never escalated.
- **Cost-decorrelated perception throttle bucketing** *(perception)* — **made its own target phase slower (+0.48 ms vs 0.09 ms noise).** The scramble + keyed-gate overhead plus worse cache locality cost more than spreading the expensive-giant tail across ticks saves. Wall delta −0.08 ms is essentially zero. **Hypothesis not supported.** Triage-only.
- **Hoist max_accel divide + defer SteeringContext build** *(steering)* — a sound micro-op, but its steering-phase delta (−0.34 ms) sits *on* the 0.34 ms noise floor at 1M: no reliable per-phase win, and worst-phase regression was +0.04 ms. Below the detection bar at scale. Bundled determinism test is a good guard if revisited.
- **Overlap update_body_size_cache with perception** *(behavior, barrier-variance lever)* — the reorder is correct and bit-identical (good tests), and the phase win (−0.22 ms) just clears its 0.13 ms floor, but the wall win (−0.78 ms) is *below* the 0.91 ms wall noise floor — not a credible end-to-end gain. The riskiest idea on the slate (an incorrect reorder silently corrupts physics) for a payoff that vanishes into noise at 1M.

---

## Recommend merging

1. **Parallelize L1 aggregation** — the only confirmed win: **−2.95 ms wall p99** with **zero behavior change**, bit-identical, test-guarded, and safe (gate auto-ditches if fork overhead ever flips it). Merge it. **One condition:** encapsulate/assert the "L0 immutable during aggregation" invariant so a future edit can't silently break soundness.

That's the whole shortlist — one clean, high-confidence Keep.

## What to hunt next

- **Go where the time is: steering (p99 13.56 ms) is the dominant phase** and remains untouched. The two micro-ops tried this run (max_accel hoist, ctx defer) were below the noise bar — the next steering hunt needs an *algorithmic* lever (e.g. cut per-neighbor force work, restructure the steering neighbor loop, or SoA the steering inputs), not a 1–2-op trim.
- **Perception (p99 10.08 ms) resisted two angles this run** (circular cone bound, cost-decorrelated bucketing) — both *regressed* the phase. Treat further per-cell perception micro-opts as suspect; look instead at reducing how many cells are queried (`cells_queried` p99 ≈ 2.75M is huge) via tighter culling that doesn't add per-column transcendentals.
- **Movement (p99 9.45 ms)** is the third-largest phase and was not probed this run — a fresh target.
