export const meta = {
  name: 'perf-hunt',
  description: 'Hunt, implement, measure & rank Speciate perf optimizations through the phase-aware lab gate; learns via a ledger',
  whenToUse: 'When you want to automatically discover and validate performance optimizations. Pass a number = how many ideas to hunt.',
  phases: [
    { title: 'Recall', detail: 'read the ledger + checklist; brief the hunters' },
    { title: 'Ideate', detail: 'parallel hunter fleet proposes ideas; synthesizer dedupes vs ledger' },
    { title: 'Implement', detail: 'each idea built in an isolated worktree, gated on cargo test -> unified diff' },
    { title: 'Measure', detail: 'SERIAL: one lab run at a time, A/B vs baseline through classify()' },
    { title: 'Accumulate', detail: 'SERIAL: stack DEFER wins, measure the union' },
    { title: 'Report', detail: 'rank, log verdicts to the ledger, present tradeoffs' },
  ],
}

// ---- config: bare number = idea count, or an object -------------------------
// args can arrive as a number (/perf-hunt N), an object, or a JSON string — normalize all.
let cfg = args
if (typeof cfg === 'string') {
  try { cfg = JSON.parse(cfg) } catch (e) { const n = parseInt(cfg, 10); cfg = Number.isFinite(n) ? { count: n } : {} }
}
if (typeof cfg === 'number') cfg = { count: cfg }
if (!cfg || typeof cfg !== 'object') cfg = {}
const COUNT = cfg.count || 5
const PILOT = !!cfg.pilot
// 1M is the realistic-DNA wall (≈48.5ms mean / 49.4 p99 of the 50ms budget) — the
// stress point where the tail actually bites, so that's where headroom is measured.
const FULL_POP = cfg.fullPop || (PILOT ? 200000 : 1000000)
const TRIAGE_POP = cfg.triagePop || (PILOT ? 80000 : 500000)
const SEEDS = cfg.seeds || (PILOT ? '11,42,99' : '11,42,99,137,2025')
const TRIAGE_SEEDS = cfg.triageSeeds || '11,42,99'

const REPO = 'F:/dev/speciate'
const SIM = REPO + '/apps/simulation'
const LEDGER = REPO + '/docs/scale/perf-hunt/ledger.jsonl'
const ART = SIM + '/target/perf-hunt' // inside target/ => gitignored
const WORLD = '--realistic-dna --half-x 5000 --half-y 5000'
// EFFICIENCY (not a load cap): point every worktree at ONE shared, pre-warmed target
// dir so Bevy's dependencies compile exactly once instead of N times; cargo's own build
// lock then coordinates concurrent builds — they queue, they don't corrupt. Build
// parallelism is UNCAPPED by default (use all cores): the 2026-06-24 hard-off was an
// unstable PBO undervolt, not a workflow fault. Pass cfg.jobs to re-impose a -j cap if a
// machine ever proves unstable under load.
const JOBS = cfg.jobs // undefined => uncapped (all cores)
const JOBS_FLAG = JOBS ? (' -j ' + JOBS) : ''
const SHARED_TARGET = SIM + '/target'
const BUILD = 'cargo build' + JOBS_FLAG + ' --release --features dev-tools --bin latency_lab --target-dir ' + SHARED_TARGET
const RUN = 'cargo run --quiet' + JOBS_FLAG + ' --release --features dev-tools --target-dir ' + SHARED_TARGET + ' --bin latency_lab --'
// Back-to-back A/B (RCA 2026-06-24): the candidate is judged against a freshly-run
// CLEAN baseline binary stashed once, NOT a stale pinned baseline — kills drift, and
// a clean-tree guard kills cross-candidate patch contamination.
const RELEASE_BIN = SHARED_TARGET + '/release/latency_lab.exe' // freshly-built candidate
const BASELINE_BIN = ART + '/latency_lab_baseline.exe' // stashed clean baseline

const SCOPE_NOTE =
  'Scope allowed: engine micro-opts (behavior-preserving), architectural/risky (schedule overlap, ' +
  'fork-join removal, SoA splits, parallelism), and Golden-Zone biological (perf that IS a gameplay ' +
  'change, e.g. hunger-gated perception). EVERY biological idea MUST state its behavior tradeoff and a ' +
  'trophic-canary plan (apex + grazer counts within +-20%) or it is rejected.'

const PHASES = 'perception | steering | movement | grid_rebuild | l1_aggregation | behavior'

// ---- schemas ----------------------------------------------------------------
const IDEA_FIELDS = {
  id: { type: 'string', description: 'stable kebab slug, unique' },
  title: { type: 'string' },
  hypothesis: { type: 'string', description: 'why this should cut time, mechanistically' },
  scope: { type: 'string', enum: ['engine', 'architectural', 'biological'] },
  target_phase: { type: 'string', enum: ['perception', 'steering', 'movement', 'grid_rebuild', 'l1_aggregation', 'behavior'] },
  files: { type: 'string', description: 'file:line anchors the change touches' },
  sketch: { type: 'string', description: 'concrete implementation sketch a Rust dev can follow' },
  predicted_wall_ms: { type: 'number', description: 'predicted wall p99 delta (negative = faster)' },
  tradeoffs: { type: 'string', description: 'cost/consequence; REQUIRED non-empty for biological' },
  canary_needed: { type: 'boolean' },
}
const IDEAS_SCHEMA = {
  type: 'object',
  properties: { ideas: { type: 'array', items: { type: 'object', properties: IDEA_FIELDS, required: ['id', 'title', 'hypothesis', 'scope', 'target_phase', 'sketch', 'tradeoffs'] } } },
  required: ['ideas'],
}
const IMPL_SCHEMA = {
  type: 'object',
  properties: {
    id: { type: 'string' },
    compiles: { type: 'boolean' },
    tests_pass: { type: 'boolean' },
    diff: { type: 'string', description: 'unified git diff of the change, empty if it failed' },
    summary: { type: 'string' },
    notes: { type: 'string' },
  },
  required: ['id', 'compiles', 'tests_pass', 'diff', 'notes'],
}
const MEASURE_SCHEMA = {
  type: 'object',
  properties: {
    id: { type: 'string' },
    verdict: { type: 'string', enum: ['KEEP', 'DEFER', 'DITCH', 'ERROR'] },
    target_phase: { type: 'string' },
    dwall_p99_ms: { type: 'number' },
    dphase_ms: { type: 'number' },
    phase_noise_ms: { type: 'number' },
    wall_noise_ms: { type: 'number' },
    triage_only: { type: 'boolean' },
    notes: { type: 'string' },
  },
  required: ['id', 'verdict', 'notes'],
}

// ---- helpers ----------------------------------------------------------------
const gateBrief =
  'THE GATE (latency_lab --verdict runs the tested classify()): improvements are judged on the per-seed ' +
  'MEDIAN, paired seed-for-seed (it is ~3-19× quieter run-to-run than p99). DETECT against the targeted ' +
  'phase own median noise floor (Δphase_median ≤ −2×phase_noise). BANK against the wall median noise floor: ' +
  'KEEP if Δwall_median ≤ −2×wall_noise; DEFER if real-at-phase but within wall noise (parked for stacking, ' +
  'NOT discarded); DITCH if the wall median regresses or any phase p99 regresses >2ms (p99 is the strict ' +
  'tail/SLO guard, NOT the detector). The VERDICT line prints dPhaseMedian/dWallMedian. No flat 3ms floor.'

// ============================================================================
phase('Recall')
const brief = await agent(
  'You are briefing a fleet of performance-optimization hunters for the Speciate Rust/Bevy ECS engine.\n' +
  'READ, in this order:\n' +
  '1. ' + LEDGER + ' (JSONL: every idea already tried + its verdict; DO_NOT_REVISIT and DONE are HARD exclusions. ' +
  'An entry with a "retest" field is RE-ELIGIBLE despite a DITCH verdict — it was ditched under the OLD noisier gate ' +
  'and the "retest" note says why it may now pass; surface these as PRIORITY re-tests, with their prior result as context)\n' +
  '2. ' + REPO + '/docs/scale/optimization-checklist.md (the Pass bar, attack order, ditched list, next levers)\n' +
  '3. ' + REPO + '/docs/scale/path-to-one-million.md (current phase budget breakdown; which phase is fattest)\n' +
  '4. Skim the hot systems: apps/simulation/src/simulation/{perception,steering,movement}/ and spatial/.\n\n' +
  'Produce a concise BRIEF (markdown, ~300 words) for the hunters covering: (a) the current per-phase time budget at ' +
  FULL_POP + ' and which phases are fattest, (b) ideas already tried and their verdicts — explicitly the ones they MUST NOT ' +
  're-propose, (c) the most promising untried levers per the checklist (T2.5, T3.1, T2.2, etc.), (d) ' + gateBrief,
  { label: 'recall-ledger', phase: 'Recall' }
)

// ============================================================================
phase('Ideate')
const ANGLES = [
  { key: 'micro', lens: 'engine micro-optimizations: cache layout, allocation, branch elimination, SoA access, reducing per-entity work in the hot loop — behavior-preserving only' },
  { key: 'arch', lens: 'architectural/structural changes: Bevy schedule overlap, removing fork-join barriers, parallelizing serial phases (L1 aggregation), proxy SoA splits — higher ceiling, needs correctness care' },
  { key: 'bio', lens: 'Golden-Zone biological levers where the perf win IS a gameplay mechanic: hunger-gated perception, giants ignoring tiny entities, satiated-predator rest, reaction-time jitter — each MUST carry tradeoffs + a trophic canary' },
]
const perHunter = Math.max(2, Math.ceil(COUNT / 2))
const proposals = (await parallel(ANGLES.map((a) => () =>
  agent(
    'You are a Speciate performance hunter focused on ' + a.lens + '.\n\nBRIEF:\n' + brief + '\n\n' + SCOPE_NOTE + '\n\n' +
    'Propose up to ' + perHunter + ' DISTINCT, concrete optimization ideas in your lens. Each needs a real implementation ' +
    'sketch with file:line anchors (read the actual code to ground it). Do NOT propose anything on the ledger DO_NOT_REVISIT/DONE list. ' +
    'Ledger entries with a "retest" field ARE fair game — re-propose them (the gate that ditched them was too noisy); reuse the prior ' +
    'sketch and cite the entry. Be honest about predicted impact and tradeoffs.',
    { label: 'hunt:' + a.key, phase: 'Ideate', schema: IDEAS_SCHEMA }
  )
))).filter(Boolean).flatMap((r) => r.ideas || [])

const synth = await agent(
  'You are the lead engineer triaging ' + proposals.length + ' proposed perf optimizations for Speciate.\n\n' +
  'PROPOSALS (JSON):\n' + JSON.stringify(proposals) + '\n\n' +
  'Cross-check every proposal against the ledger ' + LEDGER + ' — DROP anything that duplicates a DO_NOT_REVISIT/DONE/' +
  'already-DITCHED entry, EXCEPT entries carrying a "retest" field: those are explicitly re-eligible (ditched under the old ' +
  'noisier gate) and should be PREFERRED, not dropped. Then select the ' + COUNT + ' STRONGEST and most DIVERSE ideas (spread across phases and scopes, ' +
  'not all perception). Merge near-duplicates. Keep the best implementation sketch for each. Return exactly ' + COUNT +
  ' ideas (or fewer if not enough survive), each with a unique id.',
  { label: 'ideate:synthesize', phase: 'Ideate', schema: IDEAS_SCHEMA }
)
const ideas = (synth?.ideas || []).slice(0, COUNT)
log('Ideate: ' + proposals.length + ' proposed -> ' + ideas.length + ' selected: ' + ideas.map((i) => i.id).join(', '))

// ============================================================================
phase('Implement')
// Pre-warm the shared cache once so the parallel implement builds reuse compiled
// dependencies instead of each cold-compiling all of Bevy. Also a cheap canary —
// if a build can't even complete, abort before fanning out.
log('Pre-warming shared build cache...')
const warm = await agent(
  'Pre-warm the shared Rust build cache. Work in ' + SIM + '. Run:\n' + BUILD + '\nThis compiles the dependencies into the ' +
  'shared target dir so later builds are incremental. Report ok=true if it finishes, with the wall time in notes. If it fails ' +
  'to build, report ok=false with the error.',
  { label: 'prewarm-build', phase: 'Implement', schema: { type: 'object', properties: { ok: { type: 'boolean' }, notes: { type: 'string' } }, required: ['ok', 'notes'] } }
)
if (!warm || !warm.ok) {
  log('Pre-warm failed (' + (warm?.notes || 'agent died') + ') — aborting before fan-out.')
  return { aborted: 'prewarm-failed', notes: warm?.notes || 'prewarm agent returned null' }
}

// Parallel implement: agents work concurrently in isolated worktrees. They all build
// into the shared, pre-warmed target dir, so cargo's build lock serializes the actual
// compiles (deps already cached) while the thinking/editing overlaps.
const built = (await parallel(ideas.map((idea) => () =>
  agent(
    'Implement ONE performance optimization in the Speciate engine, in your isolated worktree.\n\n' +
    'IDEA: ' + JSON.stringify(idea) + '\n\n' +
    'Steps:\n' +
    '1. Read the files in the sketch and implement the MINIMAL change that realizes the hypothesis. Behavior-preserving ' +
    'unless scope is biological (then the behavior change must match the stated design).\n' +
    '2. Do NOT touch apps/simulation/src/bench_lab/ or src/bin/latency_lab.rs (the harness must stay constant).\n' +
    '3. From ' + SIM + ', run `cargo test' + JOBS_FLAG + ' --target-dir ' + SHARED_TARGET + '` (default features). It MUST ' +
    'pass — fix your change until it does. A "Blocking waiting for file lock on build directory" message is NORMAL — another ' +
    'agent is compiling into the shared target; just wait for it.\n' +
    '4. Capture the change as a unified diff: from ' + REPO + ' run `git diff` and put the FULL output in the diff field.\n' +
    '5. CLEANUP — REQUIRED: once the diff is captured in the field, restore this worktree to pristine so it auto-removes. ' +
    'From ' + REPO + ' run `git reset --hard && git clean -fd`. Your edits are already saved in the diff field and applied ' +
    'elsewhere by the harness; a dirty worktree leaks on disk and must not survive this agent.\n\n' +
    'Return compiles/tests_pass honestly. If you cannot make it compile or pass tests, return compiles/tests_pass=false, ' +
    'diff="" and explain why in notes (still run step 5). id MUST equal "' + idea.id + '".',
    { label: 'impl:' + idea.id, phase: 'Implement', schema: IMPL_SCHEMA, isolation: 'worktree' }
  )
))).filter(Boolean)
const ready = built.filter((b) => b.compiles && b.tests_pass && b.diff && b.diff.trim().length > 0)
log('Implement: ' + ready.length + '/' + ideas.length + ' compiled + passed tests: ' + ready.map((b) => b.id).join(', '))

// ============================================================================
// SERIAL from here: one sim at a time on a quiet machine (noisy-neighbour).
phase('Measure')
const idById = Object.fromEntries(ideas.map((i) => [i.id, i]))

// Build ONCE on the clean (committed) tree and STASH the binary, so every candidate
// can run it back-to-back without a rebuild. A back-to-back pair cancels machine drift.
const baseline = await agent(
  'Build and STASH the clean baseline binary for back-to-back A/B. Work in ' + SIM + ' (NO worktree); the tree is the ' +
  'committed state — do not modify source.\n\n' +
  '1. mkdir -p ' + ART + '\n' +
  '2. CLEAN-TREE GUARD: from ' + REPO + ', run `git diff --quiet -- apps/simulation/src`. If it exits non-zero, source is dirty — return ok=false and STOP.\n' +
  '3. Build: ' + BUILD + '\n' +
  '4. Stash the binary so candidates run it without rebuilding: cp ' + RELEASE_BIN + ' ' + BASELINE_BIN + '\n' +
  '5. Sanity run + record the number: ' + BASELINE_BIN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + '\n\n' +
  'Return ok=true and the baseline wall mean-of-p99s (ms) in notes. This run owns the machine; nothing else is running.',
  { label: 'measure:baseline-stash', phase: 'Measure', schema: { type: 'object', properties: { ok: { type: 'boolean' }, wall_p99_ms: { type: 'number' }, notes: { type: 'string' } }, required: ['ok', 'notes'] } }
)
if (!baseline || !baseline.ok) {
  log('Baseline build/stash failed (' + (baseline?.notes || 'agent died') + ') — aborting measure.')
  return { aborted: 'baseline-failed', notes: baseline?.notes || 'baseline agent returned null' }
}

const measured = []
for (const cand of ready) {
  const idea = idById[cand.id] || {}
  const phaseName = idea.target_phase || 'perception'
  const m = await agent(
    'Measure ONE candidate via a CLEAN BACK-TO-BACK A/B. SERIAL — you own the machine; never background a run.\n\n' +
    'Candidate id: ' + cand.id + ' (targets phase: ' + phaseName + ')\n' +
    'Diff between the <DIFF> markers:\n<DIFF>\n' + cand.diff + '\n</DIFF>\n\n' +
    'The stashed CLEAN baseline binary is ' + BASELINE_BIN + ' — run it as-is, never rebuild it. Protocol (from ' + REPO + '):\n' +
    '1. CLEAN-TREE GUARD: `git diff --quiet -- apps/simulation/src`. If it exits non-zero the tree is contaminated by a prior ' +
    "candidate's failed revert — return verdict=ERROR, notes=\"dirty tree before apply\", and STOP. NEVER measure on a dirty tree.\n" +
    '2. Write the diff to ' + ART + '/' + cand.id + '.patch; `git apply --check` it; if it does not apply cleanly return verdict=ERROR and STOP.\n' +
    '3. `git apply ' + ART + '/' + cand.id + '.patch`; build candidate: (cd ' + SIM + ' && ' + BUILD + '). If build fails, `git apply -R` and return verdict=ERROR. The candidate binary is now ' + RELEASE_BIN + '.\n' +
    '4. TRIAGE — back-to-back, BASELINE FIRST then candidate (adjacent runs cancel drift):\n' +
    '   ' + BASELINE_BIN + ' --seeds ' + TRIAGE_SEEDS + ' --pop ' + TRIAGE_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.base.triage.json\n' +
    '   ' + RELEASE_BIN + ' --seeds ' + TRIAGE_SEEDS + ' --pop ' + TRIAGE_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.triage.json\n' +
    '   verdict: ' + RELEASE_BIN + ' --verdict --baseline ' + ART + '/' + cand.id + '.base.triage.json --candidate ' + ART + '/' + cand.id + '.triage.json --phase ' + phaseName + '\n' +
    '   If triage VERDICT=Ditch AND dWallP99 is not clearly negative, set triage_only=true, use the triage verdict, and skip to step 6.\n' +
    '5. ESCALATE — back-to-back at full pop, BASELINE FIRST then candidate:\n' +
    '   ' + BASELINE_BIN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.base.full.json\n' +
    '   ' + RELEASE_BIN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.full.json\n' +
    '   verdict: ' + RELEASE_BIN + ' --verdict --baseline ' + ART + '/' + cand.id + '.base.full.json --candidate ' + ART + '/' + cand.id + '.full.json --phase ' + phaseName + '\n' +
    '6. REVERT + GUARD: `git apply -R ' + ART + '/' + cand.id + '.patch`, then `git diff --quiet -- apps/simulation/src` MUST pass. If it does not, note "revert FAILED — tree dirty" so the next candidate aborts.\n\n' +
    'Convert microseconds to ms (÷1000). Return the verdict (the escalated full-pop one if you ran it), dwall_p99_ms, dphase_ms, ' +
    'phase_noise_ms, wall_noise_ms, triage_only, and notes. id MUST equal "' + cand.id + '".',
    { label: 'measure:' + cand.id, phase: 'Measure', schema: MEASURE_SCHEMA }
  )
  if (m) measured.push({ ...m, scope: idea.scope, title: idea.title, tradeoffs: idea.tradeoffs, target_phase: m.target_phase || phaseName })
}

// REPLICATE every KEEP once more, back-to-back, before trusting it. A win that does
// not reproduce is demoted to DITCH. (RCA 2026-06-24: a single contaminated measurement
// turned a +10ms regression into a phantom -3ms KEEP that shipped.)
for (const k of measured.filter((m) => m.verdict === 'KEEP')) {
  const phaseName = k.target_phase || 'perception'
  const rep = await agent(
    'REPLICATE a KEEP to confirm it reproduces (RCA 2026-06-24 guard). SERIAL. From ' + REPO + ':\n' +
    '1. CLEAN-TREE GUARD: `git diff --quiet -- apps/simulation/src`; if dirty return verdict=ERROR and STOP.\n' +
    '2. Re-apply the patch already on disk: `git apply ' + ART + '/' + k.id + '.patch`; build (cd ' + SIM + ' && ' + BUILD + ').\n' +
    '3. Back-to-back full A/B, BASELINE FIRST:\n' +
    '   ' + BASELINE_BIN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/' + k.id + '.rep.base.json\n' +
    '   ' + RELEASE_BIN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/' + k.id + '.rep.json\n' +
    '   verdict: ' + RELEASE_BIN + ' --verdict --baseline ' + ART + '/' + k.id + '.rep.base.json --candidate ' + ART + '/' + k.id + '.rep.json --phase ' + phaseName + '\n' +
    '4. REVERT + GUARD: `git apply -R ' + ART + '/' + k.id + '.patch`; `git diff --quiet -- apps/simulation/src` must pass.\n\n' +
    'Return the replication verdict, dwall_p99_ms, and notes. id MUST equal "' + k.id + '".',
    { label: 'replicate:' + k.id, phase: 'Measure', schema: MEASURE_SCHEMA }
  )
  // Demote any KEEP whose replication did not also land KEEP.
  if (!rep || rep.verdict !== 'KEEP') {
    k.notes = '[NOT REPRODUCED on replication: ' + (rep ? rep.verdict + ', dWall=' + rep.dwall_p99_ms + 'ms' : 'agent died') + '] ' + (k.notes || '')
    k.verdict = 'DITCH'
    k.replicated = false
  } else {
    k.replicated = true
    k.notes = '[replicated: dWall=' + rep.dwall_p99_ms + 'ms] ' + (k.notes || '')
  }
}

// ============================================================================
phase('Accumulate')
const keeps = measured.filter((m) => m.verdict === 'KEEP')
const defers = measured.filter((m) => m.verdict === 'DEFER')
let bundle = null
if (defers.length >= 2) {
  const diffs = defers.map((d) => ({ id: d.id, patch: ART + '/' + d.id + '.patch', phase: d.target_phase }))
  bundle = await agent(
    'Test whether stacking these DEFER wins clears the bank bar as a UNION. SERIAL — you own the machine.\n\n' +
    'Patches to stack (already on disk from the measure phase): ' + JSON.stringify(diffs) + '\n\n' +
    'Protocol (from ' + REPO + '). The stashed clean baseline binary is ' + BASELINE_BIN + ' (run as-is):\n' +
    '1. CLEAN-TREE GUARD: `git diff --quiet -- apps/simulation/src`; if dirty return verdict=ERROR and STOP.\n' +
    '2. Apply each patch in turn with `git apply` (check each first). If any conflicts with an already-applied one, skip it and note which.\n' +
    '3. Build the union: (cd ' + SIM + ' && ' + BUILD + ').\n' +
    '4. Back-to-back full A/B, BASELINE FIRST then union:\n' +
    '   ' + BASELINE_BIN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/bundle.base.json\n' +
    '   ' + RELEASE_BIN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/bundle.full.json\n' +
    '   verdict (dominant phase = the largest individual phase win among the stacked defers): ' +
    RELEASE_BIN + ' --verdict --baseline ' + ART + '/bundle.base.json --candidate ' + ART + '/bundle.full.json --phase <dominant_phase>\n' +
    '5. Reverse every applied patch (`git apply -R`) and confirm `git diff --quiet -- apps/simulation/src` passes.\n\n' +
    'Return id="bundle", the verdict, deltas in ms, and in notes: which patches actually stacked, and whether the union delivered ' +
    'roughly the SUM of the individual defer wins (additive) or under-delivered (a sign two changes fight over the same resource).',
    { label: 'accumulate:union', phase: 'Accumulate', schema: MEASURE_SCHEMA }
  )
}

// ============================================================================
phase('Report')
const reportInput = {
  config: { count: COUNT, pilot: PILOT, full_pop: FULL_POP, triage_pop: TRIAGE_POP, seeds: SEEDS },
  ideas_considered: ideas.length,
  implemented: ready.length,
  baseline_notes: baseline?.notes,
  measured,
  keeps,
  defers,
  bundle,
}
const report = await agent(
  'Write the final Perf Hunt report for a human decision-maker, and update the learning ledger.\n\n' +
  'RESULTS (JSON):\n' + JSON.stringify(reportInput) + '\n\n' +
  'PART A — LEDGER: append one JSONL line per measured candidate AND the bundle to ' + LEDGER + '. Get today date via ' +
  '`date +%Y-%m-%d`. Fields: id, date, title, scope, target_phase, verdict, dwall_p99_ms, dphase_ms, notes. ' +
  'Do not rewrite existing lines; append only.\n\n' +
  'PART B — REPORT: write a compelling, skimmable markdown report to ' + REPO + '/docs/scale/perf-hunt/last-run.md with:\n' +
  '- A headline summary table: each idea | scope | target phase | verdict | Δwall p99 (ms) | Δphase (ms).\n' +
  '- KEEPS first, then any winning BUNDLE (was it additive?), then DEFERS (parked), then DITCHED (with why).\n' +
  '- For every KEEP and the bundle: the explicit TRADEOFF / consequence (especially biological ones — the behavior change ' +
  'and the trophic-canary result). The human is choosing what to merge; make the cost legible.\n' +
  '- A short "recommend merging" shortlist with one-line justifications, and what to hunt next.\n\n' +
  'Return a tight plain-text executive summary (the same headline table + the merge shortlist) as your final message.',
  { label: 'report+ledger', phase: 'Report' }
)

return {
  ideasConsidered: ideas.length,
  implemented: ready.length,
  keeps: keeps.map((k) => ({ id: k.id, dwall_ms: k.dwall_p99_ms })),
  defers: defers.map((d) => ({ id: d.id, dwall_ms: d.dwall_p99_ms })),
  bundle: bundle ? { verdict: bundle.verdict, dwall_ms: bundle.dwall_p99_ms } : null,
  reportPath: REPO + '/docs/scale/perf-hunt/last-run.md',
  summary: report,
}
