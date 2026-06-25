# 🚧 Perf Hunt — Last Run (2026-06-25)

**Target:** 1,000,000 creatures, 20 Hz (50 ms tick budget). Decision metric is **wall p99**, measured 5-seed (11,42,99,137,2025), realistic-DNA, baseline-first back-to-back A/B through the phase-aware latency lab.

## Baseline this run

| Metric | Value |
|---|---|
| Wall p99 (mean-of-p99s) | **44.663 ms** (of 50 ms budget) |
| Wall noise floor (σ) | 0.257 ms |
| Wall mean | 42.965 ms |

**Per-phase p99 (ms):** steering **13.635** · movement **9.601** · perception 7.512 · grid_rebuild 6.887 · behavior 4.671 · l1_aggregation 4.381.

> Fattest phase is **steering at 13.6 ms (~30% of the wall budget)** — and that is where this hunt concentrated. We are ~10% of a tick budget from the 1M stretch target; nothing this run cleared the bank bar.

---

## Headline: nothing merged

5 ideas considered, 5 implemented, **0 KEEPs**. 2 parked as DEFER, 2 DITCHED, 1 failed to build. The bundle did not stack.

| Idea | Scope | Target phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|---|---|---|---|---:|---:|
| Movement: atan2-free turn-limit (dot/cross) | engine | movement | **DEFER** | −0.506 | −0.347 |
| Movement: kill powf(1.0) + quartic max_speed | engine | movement | **DEFER** | +0.703 | −0.272 |
| Bundle (kill-powf + atan2-free) | — | movement | **DEFER** (collapsed) | +0.143 | −0.296 |
| Shared-collect two-pass steer/integrate | architectural | steering | **DITCH** (triage) | +0.482 | −1.926 |
| Golden-Zone: giants ignore mice (size dominance) | biological | steering | **DITCH** (triage) | −0.125 | −0.050 |
| Native Bevy par_iter steering+movement | architectural | steering | **ERROR** (no build) | — | — |

*Δphase is the verdict tool's median delta; Δwall is dWallMedian (the lab reports a median wall delta, not a separate p99 wall figure). Negative = faster.*

---

## ✅ KEEPS

**None this run.** No candidate produced a wall-p99 improvement that cleared its noise floor at full pop.

---

## 📦 BUNDLE — did NOT stack (collapsed)

The two surviving DEFERs were union-tested. **Only 1 of 2 patches applied** — they are **mutually exclusive as authored**: both rewrite the *same* turn-rate-limiter block in `integrate_motion_system` (`apps/simulation/src/simulation/movement/systems.rs` ~L160–196) and both edit the same math `use` import (L9). `atan2-free` was authored against the clean baseline, so it cannot machine-merge onto the `kill-powf` tree (`git apply --check` and `--3way` both fail).

> The conflict **is** the answer: these are not additive — they compete for the exact same lines. Shipping both requires a hand-merge into a single rewritten loop.

The union that did run (kill-powf only, full 5-seed 1M): movement phase −0.296 ms (~2.4× its 0.124 ms noise — real but small), but **wall p99 +0.143 ms against 0.553 ms noise** — no wall win outside noise. **Bank bar not cleared.**

---

## 🅿️ DEFERS (parked — phase-real, wall-inconclusive)

Both target **movement (9.6 ms)** and both shave the phase repeatably, but the saving is **smaller than wall noise at 1M**, so neither is distinguishable from drift end-to-end. Parked for a future *stacked* movement-loop rewrite (see "hunt next").

### 1. Movement: atan2-free turn-rate limiting — Δphase −0.347 ms, Δwall −0.506 ms
Replaces 2 `fast_atan2` calls on the turn-limit hot path with dot/cross + a single `sincos` only on the clamp branch. Phase win is real and repeatable at both pops (~0.35 ms, well above 0.118 ms phase noise). Wall delta is negative (−0.506 ms) but **inside** the 0.683 ms wall noise → Defer, not Keep.

> **TRADEOFF — NOT bit-identical.** Clamped headings now come from a direct rotation instead of atan2/cos/sin composition, so numerics shift slightly. **Requires a determinism canary** (cells_queried identical + heading distribution within tolerance across all 5 seeds) before merge. Not a trophic canary — behavior *intent* is unchanged. Ships unit tests proving `limit_turn` matches the old atan2 reference within 1e-4 and leaves the no-clamp velocity path untouched.

### 2. Movement: kill powf(1.0) + quartic max_speed — Δphase −0.272 ms, Δwall +0.703 ms
Replaces `powf(0.25)` with `sqrt().sqrt()` and folds `powf(1.0)` to identity in the turn-rate base. Phase win real and consistent (−0.340 then −0.272 ms), but **wall p99 REGRESSED +0.703 ms** against 0.521 ms noise (~1.35× — not clearly real either direction). Phase-local win that doesn't carry to wall.

> **TRADEOFF.** The turn-rate divide is **bit-identical, zero behavior risk**. The `max_speed` `sqrt().sqrt()` half perturbs results at ~1 ULP and could trip a strict spec expecting the exact `powf` value — **run the determinism canary before merging that half only.**

---

## 🗑️ DITCHED (with why)

### Shared-collect two-pass steer/integrate — Δwall +0.482 ms, Δphase −1.926 ms *(triage-only)*
The single biggest **phase** mover of the run: deletes a redundant 1M gather+reload, cutting ~3 ms across the two fused passes (triage steering p99 6.48→4.72 ms, movement 4.23→2.90 ms). **But wall p99 regressed** (base 21.7 ms vs cand 23.7 ms) with much higher candidate noise (σ 1.82 ms vs 0.38 ms). The per-phase savings **did not propagate to end-to-end latency**, so per protocol (Ditch + dWall not clearly negative) it was ditched at triage with no escalation.

> **Diagnostic value:** this variant recovers the collect+reload but **retains the fork-join barrier**. Its failure to move wall isolates the **barrier**, not the redundant collect, as the real cost behind the previously-held −2.7 ms fuse. That's the load-bearing finding for any future steering-fuse attempt.

### Golden-Zone: giants ignore mice — Δwall −0.125 ms, Δphase −0.050 ms *(triage-only)*
A size-dominance early-out in the avoidance neighbor loop. **Both deltas sat at/under the noise floor** (phase 0.051 ms noise, wall 0.118 ms noise) — no measurable win — so escalation was skipped. Confirms **ledger #15** (avoidance-sqrt-free): per-neighbor branches **wash in dense convergent packs**; the win only materializes when real size disparity is present and even then didn't clear noise here.

> **TRADEOFF (would have been a BEHAVIOR CHANGE, not bit-identical).** Large creatures stop deflecting around much smaller ones — intended size-dominance/trample pressure, but it removes a survival buffer for the smallest grazers, and `DOMINATION_RATIO` is a magic number that must migrate to DNA. **No trophic canary was run** (ditched before merge consideration). If ever revisited: must run under `--realistic-dna` with the trophic canary (apex & grazer populations each within ±20%) and log the gene range + ratio in `docs/biology/biology-notes.md`.

### Native Bevy par_iter steering+movement — ERROR (no build)
`cargo` exit 101: the diff references `bevy_ecs::query::BatchingStrategy`, which **does not exist at that path** in bevy_ecs 0.14.2 (E0433 at `steering/system.rs:170` and `movement/systems.rs:62`). In 0.14 the type is re-exported elsewhere. No measurement. Patch reverted clean.

> **Fix-forward note:** re-author against the correct 0.14.2 import path before re-attempting — the idea (drop the serial 1M collect for Bevy-native parallel iteration) is untested, not refuted.

---

## ⭐ Recommend merging — shortlist

**Nothing meets the bank bar this run.** Honest call: **merge nothing.** No candidate produced a wall-p99 win outside its noise floor at 1M.

Closest-to-mergeable, if a movement-loop pass is opened deliberately:
- **`movement-atan2-free-turn-limit`** — the only candidate with both phase *and* wall deltas pointing the right way; ships its own correctness unit tests. Merge *only* bundled into a single hand-merged movement-loop rewrite (it conflicts with kill-powf) **and after** the determinism canary passes. On its own it lands sub-noise on wall.
- **`movement-kill-powf` (turn-rate divide half only)** — bit-identical, zero-risk; safe to fold opportunistically into the same rewrite. Skip the `max_speed` half unless the determinism canary clears it.

---

## 🎯 What to hunt next

1. **Attack the fork-join barrier in steering, not the collect.** The shared-collect ditch proved the redundant 1M gather is *not* the wall cost — the barrier is. Steering is still the fattest phase (13.6 ms). Hunt barrier elimination / phase fusion that removes the join, not just the gather.
2. **Hand-merge the two movement DEFERs into one rewritten turn-limit loop and A/B the combined unit.** They're mutually exclusive as separate patches but together rewrite the whole hot block — the combined phase win (~0.3–0.6 ms) may finally clear wall noise where each half can't alone. Gate on the determinism canary.
3. **Re-author the Bevy-native par_iter idea** against the correct bevy_ecs 0.14.2 import path — it never got a measurement and directly attacks the steering serial collect.
4. Movement micro-ops are at the **noise floor**; further single-loop arithmetic wins are unlikely to bank alone. Bundle them or move up to the steering barrier for headline movement.
