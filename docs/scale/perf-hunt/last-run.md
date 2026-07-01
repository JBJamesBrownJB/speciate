# Perf Hunt Report — 2026-07-01

**Run type:** Full-fidelity home-rig retest of a cloud-triage PRIME candidate (ledger #77).
**Baseline:** 1M pop, 5 seeds (11/42/99/137/2025), realistic DNA, half-world 5000×5000, release + dev-tools `latency_lab`.
Wall p99 mean-of-p99s = **34.115 ms** (noise-floor std 1.322 ms, worst seed 36.375 ms). Wall mean-of-means = 29.706 ms.
Ideas considered: 1 | Implemented: 1 | Keeps: **0** | Defers: **1** | Ditches: 0 | Bundle: none (single candidate).

Clean-tree guard passed before and after; patch applied/built/reverted cleanly (`git diff --quiet` green post-revert). Nothing from this run is in the tree.

---

## Headline Table

| Idea | Scope | Phase | Verdict | Δwall p99 (ms) | Δphase (ms) |
|------|-------|-------|---------|----------------|-------------|
| Retest #77 (PRIME): cache counting-sort cell index in count pass, reuse in scatter | engine | grid_rebuild | **DEFER** | +1.395 † | **−0.350** |

† Reported figure is the p50-of-per-seed wall-p99 delta; the candidate run had one noisy seed (per-seed wall-p99 stdDev 4.05 ms vs baseline 1.30 ms), so it is inside noise. The verdict tool's dWallMedian was **+0.23 ms** vs wallNoise 1.70 ms — i.e. wall is **flat**, not regressed.

---

## KEEPS

None. Nothing cleared the wall-clock gate this run.

## BUNDLE

None — single candidate, nothing to stack.

---

## DEFERS (parked, real phase win, no detectable wall win)

### retest77-cache-cell-index-scatter — cache the counting-sort cell index, reuse it in scatter

**The phase win is real. The wall win is not (yet) detectable.**

- **Phase:** dPhaseMedian = **−350 µs** on grid_rebuild against a 153 µs noise floor — a clean >2× noise improvement. grid_rebuild per-seed p99 mean improved **7.714 ms → 7.601 ms**.
- **Wall:** dWallMedian = **+230 µs** inside a 1,701 µs wall noise floor — statistically flat. worstPhaseP99Regression = 793 µs (other phases jitter, nothing conclusive).
- **Triage corroborates:** 500k / 3 seeds also Defer — dPhaseMedian −125 µs (noise 52 µs), dWallMedian −32 µs (noise 91 µs). Same shape at both scales: real phase saving, wall-invisible.
- **The cloud projection did not survive contact with the home rig.** The cloud −3.3 ms @1M figure was an illustrative growth-aware extrapolation from a 10k shared-VM run; honest home-rig expectation was −0.8 to −2 ms, and the measured phase delta (−0.35 ms) landed below even that. Scatter is one of three passes in a ~6.9 ms phase and is partly atomic/memory-bound; the compute-bound fraction the win depends on is smaller at 1M than the growth-exponent improvement (0.361→0.331) suggested.

**Tradeoffs / consequences if later merged (as part of a stack):**

- **Memory:** +4–8 MB resident (one u32 per entity; ×2 if the scratch lives per-grid in the double-buffered pair).
- **Bandwidth:** one extra sequential streaming write in the count pass and one extra read stream in scatter — this is likely why the phase-local saving doesn't propagate to wall at 1M.
- **Biology / behavior: none.** Bit-identical proxy placement; behavior-preserving; no trophic impact; no canary needed. This is the safest possible kind of defer — zero gameplay risk.

**Why DEFER rather than DITCH:** the grid_rebuild improvement is real and stackable. On its own it can't move a 34 ms wall through a 1.7 ms noise floor, but combined with one or two other grid_rebuild reductions (the phase is ~7.6 ms of p99) it could cross the detection bar. Verdict-tool exit code 2 = Defer by design, not an error.

---

## DITCHED

None this run.

---

## Recommend merging

**Nothing.** One-line justifications:

- **retest77-cache-cell-index-scatter — do not merge alone.** Real −0.35 ms phase win, but wall-flat at 1M (+0.23 ms inside 1.70 ms noise) and it costs 4–8 MB resident; park it for stacking. Zero behavioral risk when it does land.

## What to hunt next

1. **Stack grid_rebuild defers.** grid_rebuild p99 is ~7.6 ms; this defer plus one more grid_rebuild win (e.g. attacking the atomic/memory-bound scatter contention directly, since compute-side savings don't propagate) could produce a detectable wall delta together.
2. **Go where the fat is: steering and perception.** They are the two fattest phases in this baseline; even the full cloud projection for #77 was smaller than a modest % win there. Detect bars: perception >1.52 ms, steering >0.95 ms.
3. **Behavior-phase noise (std 875 µs, bar >1.75 ms)** is the noisiest floor relative to phase size — candidate ideas there need to be big, or the harness needs more seeds/repeats to tighten the floor first.
4. **Recalibrate cloud→home projections.** Cloud growth-aware extrapolations overstated this win ~10×; future PRIME candidates should carry the honest home-rig range as the headline number, not the cloud figure.
