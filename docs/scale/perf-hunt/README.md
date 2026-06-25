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

- **Detect against the targeted phase's own noise floor** (`Δp99_phase ≤ −2×noise_phase`) — a real win
  in one phase isn't drowned by whole-tick noise.
- **Bank against the wall floor:** KEEP if the tick visibly moves; DEFER if real-but-tick-invisible
  (parked for stacking, never discarded as noise); DITCH if it regresses the tick or any phase >2 ms.

The noise floor is the std of the **paired** per-seed A/B differences (Common Random Numbers), not a
single arm's across-seed spread — shared world variance cancels (commit `505e591`). ⚠️ **Known limit:**
at 1M the floor is dominated by run-to-run drift (±2.3 ms wall between two identical runs), which
pairing cannot cancel — so sub-~2.3 ms wall wins are currently *unconfirmable*. See
[`noise-characterization-2026-06-25.md`](noise-characterization-2026-06-25.md) for the data and the fix
(detect on a stabler statistic than p99).

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

`DO_NOT_REVISIT` / `DONE` entries are hard exclusions — the hunters are told to skip them.
`DEFER` entries are candidates for a future accumulation round.
