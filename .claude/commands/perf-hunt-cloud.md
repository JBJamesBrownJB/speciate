---
description: "Launch the CLOUD perf-triage hunt: discover N perf ideas, measure them cheaply (function benches + per-phase A/B at ~10k + a growth-rate ramp), and log the promising ones as PRIME CANDIDATES for a full home-rig /perf-hunt. Never merges. Usage: /perf-hunt-cloud [count]"
allowed-tools:
  - Workflow
  - Bash
---

# Perf Hunt (Cloud Triage)

Launch the **perf-hunt-cloud** workflow — the Linux/cloud sibling of `/perf-hunt`. Unlike the full hunt (which
banks KEEP/DITCH verdicts from serial 1M×5-seed runs on a quiet many-core rig), this one runs cheaply on a shared
VM: it **discovers, does not validate**. Per idea it measures at **function-level** (criterion benches) and
**ECS-system/phase level** (`latency_lab` per-phase A/B at ~10k), and **ramps population to ~10k to fit the growth
exponent** (is it `O(n)` or `O(n²)`?). It then **logs promising ideas as PRIME CANDIDATES in
`docs/scale/perf-hunt/ledger.jsonl`** (as `verdict: "CANDIDATE"` with a `retest` note) so the next full `/perf-hunt`
on the home rig automatically surfaces them as priority re-tests. **It never implements/merges anything.**

**Argument:** `$ARGUMENTS` = how many ideas to hunt (integer). If empty, default to **5**.

## What to do

1. Parse `$ARGUMENTS` as an integer idea count `N` (fallback 5).
2. Resolve the repo root and launch the workflow:

   - `REPO="$(git rev-parse --show-toplevel)"`
   - `Workflow({ scriptPath: "<REPO>/.claude/workflows/perf-hunt-cloud.mjs", args: { count: N, repo: "<REPO>" } })`

   (`args` may also be a bare integer `N` — then `repo` defaults to `.`, which works when the session cwd is the repo
   root. Passing `repo` explicitly is safer.)
3. The workflow runs in the background and notifies on completion. Tell the user:
   - This is a **cheap triage run** on the cloud VM — pops top out at ~10k, so the numbers are **discovery signals,
     not authoritative verdicts**. The durable signal is the **growth exponent** (scaling class), which survives a
     noisy shared machine better than absolute wall time.
   - It **logs prime candidates to the ledger and writes `docs/scale/perf-hunt/cloud-last-run.md`** — it does **not**
     merge or bank anything.
   - The home-rig `/perf-hunt` will **auto-pick-up** those candidates (via their `retest` field) for full 1M validation.
   - They can watch live with `/workflows`.

Do not re-explain the whole pipeline unless asked — confirm the launch, the idea count, and the "triage-only, logged
for the home rig" framing.
