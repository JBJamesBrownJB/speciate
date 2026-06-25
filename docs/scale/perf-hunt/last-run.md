# 🚧 Perf Hunt — Last Run (2026-06-25)

**Headline: 0 keeps, 0 merge-ready bundle. 9 ideas hunted, all 9 DITCHED by the phase gate.**
One ditched candidate (**size-gated steering latency**) flagged a real, beyond-noise **−2.5 ms wall-tick win** that the phase timer alone couldn't credit — it is the single item worth a human look.

## Baseline (the bar every candidate had to beat)

- Population **1,000,000** · realistic DNA · world half-extent 5000×5000 · seeds `11,42,99,137,2025`.
- **Wall p99 = 49.911 ms** (mean-of-p99s; noise floor σ=1.107 ms, worst 51.311 ms). Wall mean-of-means = 46.773 ms.
- Tick budget at 20 Hz = **50 ms**. We are sitting **~0.1 ms under budget** — right at the edge, which is exactly why a clean ≥1 ms win matters.
- Per-phase p99 (us): perception **10854** · steering **14720** (fattest, unthrottled) · movement 10078 · grid_rebuild 7535 · l1_aggregation 4913 · behavior 4469.
- Triage ran at 500k (seeds 11,42,99), back-to-back baseline-first; survivors escalated to the full 1M / 5-seed run.

## Summary table

| Idea | Scope | Target phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|------|-------|--------------|---------|---------------:|------------:|
| Size-gated steering reaction latency | biological | steering | **DITCH** (wall-win flagged) | **−2.507** | −0.524 |
| T2.1 perception range hard cap (atmospheric extinction) | biological | perception | DITCH | −0.189 | −0.315 |
| Precompute mass in PerceptionProxy (kill per-scan powi(3)) | engine | perception | DITCH | −0.951 | +0.400 |
| L1 cone: per-row disc rasterization (skip corner cells) | engine | perception | DITCH | +0.613 | +0.017 |
| L1 cone: gate cell math behind occupancy check | engine | perception | DITCH | +0.670 | +0.110 |
| L1 cone: reuse L0 l1_cache to drop classify calls | engine | perception | DITCH | +1.785 | +0.592 |
| Perception proxy SoA hot/cold split | architectural | perception | DITCH | +0.427 | +0.102 |
| Drop redundant sqrt FOV cull in collect_cells_sorted_fov | engine | perception | DITCH | +1.080 | +0.327 |
| Persistent collect buffer (Windows page-fault tax) | engine | steering | DITCH | +2.093 | +0.757 |

*(Δ < 0 = faster than baseline. Every "win" here is buried inside its own noise band — see below.)*

---

## ✅ KEEPS

**None.** No candidate cleared the phase gate with a beyond-noise improvement on its targeted phase.

## 📦 BUNDLE

**None formed.** A bundle is the A/B'd union of KEEPs and surviving DEFERs; with zero of each there was nothing to stack or test. Not additive — there was nothing to add.

## 📋 DEFERS (parked for a future stack)

**None.** No candidate earned a Defer that survived escalation.

---

## ⚠️ Flagged for human review (DITCH by gate, but a real wall win)

### Size-gated steering reaction latency — "coast-on-momentum" throttle for giants
`scope: biological · phase: steering · Δwall p99 = −2.507 ms · Δphase = −0.524 ms`

This is the **one item worth merging consideration**, and the only lever this run that touched the **fattest, currently-unthrottled phase (steering, ~14.7 ms p99)**.

**Why the gate said DITCH:** the steering *phase column* improved −0.524 ms against a 0.340 ms phase-noise floor (~1.5×, below the tool's KEEP bar) and there was a trivial +0.028 ms worst-phase regression elsewhere, so the per-phase verdict was Ditch.

**Why a human should still look:** the **end-to-end wall p99 improved −2.507 ms against a 1.707 ms wall-noise floor** — a clear, beyond-noise *total-tick* win, and it was directionally consistent from triage (−0.938 ms @500k) to full (−2.507 ms @1M). The phase timer under-credits it because parallel Rayon phase boundaries can hide work that only surfaces as overall tick reduction. At a baseline sitting 0.1 ms under budget, a real −2.5 ms is the difference between "at the edge" and "comfortable headroom."

**The mechanism:** `steering_divisor = 4`, gated on `length >= 5.0 m`, so genuinely large creatures recompute steering 1-in-4 ticks and coast on momentum between.

**The TRADEOFF (the cost the human is buying):**
- **Behavioral signature (intended):** big creatures get *sluggish reflexes* — delayed collision-avoidance and seek correction. This is biologically on-theme (giant = slow to react), and is the Golden-Zone shape: the optimization *is* the feature.
- **Behavioral risk (the cost):** giants can drift into transient overlaps between updates and look less twitchy; a fleeing giant could clip an obstacle for one extra tick. Crowd separation for giants degrades.
- **Tuning hazard:** `LARGE_STEER_LEN` (5.0 m) must be set so only genuinely large creatures throttle; set too low and ordinary creatures go sluggish.
- **🐤 Trophic canary — REQUIRED before merge, NOT YET RUN:** confirm apex and grazer populations stay within ±20% over a multi-tick run, and that giants don't pile into overlaps. The −2.5 ms is a perf measurement only; the biological cost has **not** been validated this run.

---

## ❌ DITCHED (and why)

**Two biological levers — sound, but no perf payoff:**
- **T2.1 perception range hard cap (450 m atmospheric-extinction ceiling).** Escalated to 1M out of caution; both phase (−0.315 ms) and wall (−0.189 ms) deltas sat *far inside* their noise floors (0.767 ms / 2.604 ms). The cap doesn't bound enough giant L1-cones at realistic-DNA population to move the p99 tail — the tail-owning cones already fall under/near the cap or are too rare. Functionally fine, no signal.

**Five perception micro-optimizations — added overhead, didn't help:**
- **L1 cone disc rasterization (per-row x-span).** Δphase +0.017 ms within 0.299 ms noise; cells_queried essentially unchanged (1,277,254 → 1,277,263). The cone box-scan simply isn't the bottleneck at this radius — disc math is pure added cost.
- **L1 cone occupancy gate (skip empty cells).** Δphase +0.110 ms below noise. `classify_l1_cell` is already cheap enough that the extra `get_biosignature` + branch costs more than it saves.
- **L1 cone reuse L0 l1_cache.** Δwall **+1.785 ms** (clearly slower). The cache is sized for the L0 neighbor set and *misses* on the wider cone-swept L1 cells; lookup/insert overhead exceeds the saved classify calls.
- **Perception proxy SoA hot/cold split (architectural).** Δphase +0.102 ms (regression). The cold-array re-index on accept plus AoS reconstruction for low-frequency callers outweighed any reject-scan cache-density gain.
- **Drop redundant sqrt FOV cull.** Δwall **+1.080 ms**. Removing the collect-time cull *adds* cells back into the query set — cells_queried p99 rose ~1.278 M → 1.786 M (+40%); the downstream looser bitmask re-cull doesn't compensate.

**Two engine plays that backfired:**
- **Precompute mass in PerceptionProxy.** Escalated to 1M; perception phase got **worse** (+0.400 ms, above the 0.219 ms noise floor) — the extra field load / cache pressure on the proxy hot path outweighs the saved `powi(3)`. The −0.951 ms wall "win" sat inside its own 0.957 ms noise band, so it isn't credible.
- **Persistent collect buffer (Windows page-fault tax).** Δwall **+2.093 ms** — the worst regression of the run. Pre-sized `Vec::with_capacity(hwm)` + extend regressed the fattest steering phase instead of helping.

---

## Recommend merging — shortlist

1. **Nothing is merge-ready as-is.** All 9 candidates were reverted; the tree is clean.
2. **Investigate (do not auto-merge) — size-gated steering latency.** It is the only candidate showing a real, beyond-noise **−2.5 ms wall-tick win**, and it targets the fattest phase. Gate is "Ditch" only because the win surfaces at the wall, not the isolated phase timer. **Required before any merge:** run the trophic canary (apex/grazer ±20%, no giant overlap pile-ups) and tune `LARGE_STEER_LEN`. If the canary passes, this single lever buys meaningful headroom at a baseline that is 0.1 ms from budget.

## What to hunt next

- **Steering, not perception.** Steering is the fattest phase (~14.7 ms p99) and the only one still **unthrottled** — and the only win this run came from there. Eight of nine ideas chased perception and all failed; perception micro-opts are exhausted at this layout. Pivot the next batch to steering-cost reduction (size/frequency gating, force-accumulation shaving, separation-query reuse).
- **Trust the wall, instrument the phase.** This run exposed a gate gap: a real wall win read as phase-Ditch because parallel Rayon phase boundaries hide work. Worth improving how the lab attributes wall-time to phases, or adding a wall-win override path for human review.
- **Stop re-attacking the L1 cone box-scan.** Three distinct L1-cone variants this run all confirmed the box-scan is not the bottleneck at this radius (cells_queried barely moves). Park that whole family.
