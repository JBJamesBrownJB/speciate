# 🔬 Perf Hunt — automated optimization hunting + the learning ledger

> A reusable multi-agent workflow that **ideates → implements → measures → accumulates → reports**
> performance optimizations, judged by the tested phase-aware gate (`bench_lab::verdict::classify`).
> Feed it a number; it hunts that many ideas. It **remembers** every verdict so it never re-runs a dud.

## Run it

```
Workflow({ name: 'perf-hunt', args: 12 })          # hunt 12 ideas at full fidelity
Workflow({ name: 'perf-hunt', args: { count: 3, pilot: true } })   # fast pilot
```

`args` accepts a bare number (= idea count) or an object:
`{ count, pilot, fullPop, triagePop, seeds, scope }`.

## The pipeline (why each phase is parallel or serial)

| Phase | Concurrency | What happens |
|-------|-------------|--------------|
| **Recall** | 1 | Read this ledger + `optimization-checklist.md` + engine; brief the hunters on what's tried, what's hot, what not to repeat. |
| **Ideate** | parallel | Hunter fleet proposes ideas from distinct angles; a synthesizer dedupes vs the ledger to `count`. |
| **Implement** | parallel (worktrees) | Each idea implemented in isolation, gated on `cargo test`. All worktrees share one **pre-warmed `--target-dir`** so Bevy's deps compile once; cargo's build lock coordinates the concurrent compiles. Build jobs are uncapped (all cores) by default — pass `jobs` to throttle. Output = a unified diff. |
| **Measure** | **strictly serial** | One lab run at a time on a quiet machine — the sim saturates all cores, so two at once = garbage numbers (noisy neighbour). Each candidate is A/B'd vs a baseline through `classify()`. |
| **Accumulate** | **strictly serial** | DEFER wins (real but tick-invisible) are stacked and the **union** is measured. Bisect only if the union under-delivers. |
| **Report** | 1 | KEEPs, winning bundles, ceiling + tail deltas, and every tradeoff — for a human to pick what merges. Appends verdicts here. |

Nothing is auto-merged. The human is the final gate.

## The gate (what KEEP / DEFER / DITCH mean)

Encoded in `apps/simulation/src/bench_lab/verdict.rs` (`classify`), measured by
`latency_lab --verdict --baseline base.json --candidate cand.json --phase <name>`:

- **Detect on the targeted phase's median** (`Δmedian_phase ≤ −2×noise_phase`) — a real win in one
  phase isn't drowned by whole-tick noise, and the median is ~3–19× quieter run-to-run than p99.
- **Bank on the wall median:** KEEP if the tick visibly moves; DEFER if real-but-tick-invisible
  (parked for stacking, never discarded as noise); DITCH if the wall median regresses or any phase
  **p99** regresses >2 ms (p99 is the strict tail/SLO guard, not the detector).

The noise floor is the std of the **paired** per-seed A/B differences (Common Random Numbers), not a
single arm's across-seed spread — shared world variance cancels (commit `505e591`). Detection moved
from p99 to the **median** (commit `8255f67`): on a real 1M null A/B the wall noise floor fell
2340 µs → 124 µs, so sub-~2 ms wins p99 buried are now confirmable. ⚠️ **Residual:** ~120–810 µs of
run-to-run drift remains (machine heat) and needs fixed clocks / in-process A/B — see
[`noise-characterization-2026-06-25.md`](noise-characterization-2026-06-25.md).

See `docs/scale/optimization-checklist.md` for the full Pass bar.

## Ledger format (`ledger.jsonl`, append-only, one JSON object per line)

| Field | Meaning |
|-------|---------|
| `id` | stable kebab slug |
| `date` | ISO date the verdict was reached |
| `title` | one-line description |
| `scope` | `engine` \| `architectural` \| `biological` |
| `target_phase` | `perception` \| `steering` \| `movement` \| `grid_rebuild` \| `l1_aggregation` \| `behavior` |
| `verdict` | `KEEP` \| `DEFER` \| `DITCH` \| `KEEP_CORRECTNESS` \| `DONE` \| `DO_NOT_REVISIT` |
| `dwall_p99_ms` | Δ wall p99 at the test population (negative = faster) |
| `dphase_ms` | Δ targeted-phase p99 |
| `notes` | rationale, commit refs, tradeoffs |
| `retest` *(optional)* | if present, the entry is **re-eligible** despite a DITCH verdict — it was ditched under an older, noisier gate. The value says which gate change reopened it and why it may now pass. Hunters re-propose these as priority. |

`DO_NOT_REVISIT` / `DONE` entries are hard exclusions — the hunters are told to skip them.
`DEFER` entries are candidates for a future accumulation round. A `retest`-marked `DITCH` keeps its
original verdict and reason (history intact) but is surfaced for a fresh A/B under the improved gate.
