---
description: "Launch the perf-hunt workflow: hunt N performance optimizations through the phase-aware lab gate. Usage: /perf-hunt [count]"
allowed-tools:
  - Workflow
---

# Perf Hunt

Launch the reusable **perf-hunt** multi-agent workflow to discover, implement, measure, and rank
performance optimizations for the Speciate engine, judged by the tested phase-aware gate
(`bench_lab::verdict::classify`). It reads + writes `docs/scale/perf-hunt/ledger.jsonl`, so it learns
across runs and never re-proposes a ditched idea.

**Argument:** `$ARGUMENTS` = how many ideas to hunt (integer). If empty, default to **8**.

## What to do

1. Parse `$ARGUMENTS` as an integer idea count `N` (fallback 8 if missing or unparseable).
2. Launch the workflow at **full fidelity** (1M × 5-seed `--release` runs — the realistic-DNA wall) by calling:

   `Workflow({ scriptPath: "F:/dev/speciate/.claude/workflows/perf-hunt.mjs", args: N })`

   (Pass the bare integer as `args` — the script reads `typeof args === 'number'` as the count.
   Do **not** set `pilot`; the absence of it selects full 900K fidelity.)
3. The workflow runs in the background and notifies on completion. Tell the user:
   - It's a **long, serial, machine-hot run** — each candidate is measured at **1M creatures**, one
     sim at a time on purpose (noisy neighbour), so they should **stay off heavy CPU** until it
     finishes for clean numbers.
   - They can watch live with `/workflows`.
   - Nothing auto-merges — they'll get a ranked report (`docs/scale/perf-hunt/last-run.md`) with
     tradeoffs to choose what gets pushed into the engine.

Do not re-explain the whole pipeline unless asked — just confirm the launch, the idea count, and the
"stay off the CPU" note.
