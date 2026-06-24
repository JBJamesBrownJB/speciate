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
  'THE GATE (latency_lab --verdict runs the tested classify()): a change is judged by Δp99. ' +
  'DETECT against the targeted phase own noise floor (Δphase ≤ −2×phase_noise). BANK against the wall ' +
  'noise floor: KEEP if Δwall ≤ −2×wall_noise; DEFER if real-at-phase but within wall noise (parked for ' +
  'stacking, NOT discarded); DITCH if it regresses the tick or any phase >2ms. There is no flat 3ms floor.'

// ============================================================================
phase('Recall')
const brief = await agent(
  'You are briefing a fleet of performance-optimization hunters for the Speciate Rust/Bevy ECS engine.\n' +
  'READ, in this order:\n' +
  '1. ' + LEDGER + ' (JSONL: every idea already tried + its verdict; DO_NOT_REVISIT and DONE are HARD exclusions)\n' +
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
    'Be honest about predicted impact and tradeoffs.',
    { label: 'hunt:' + a.key, phase: 'Ideate', schema: IDEAS_SCHEMA }
  )
))).filter(Boolean).flatMap((r) => r.ideas || [])

const synth = await agent(
  'You are the lead engineer triaging ' + proposals.length + ' proposed perf optimizations for Speciate.\n\n' +
  'PROPOSALS (JSON):\n' + JSON.stringify(proposals) + '\n\n' +
  'Cross-check every proposal against the ledger ' + LEDGER + ' — DROP anything that duplicates a DO_NOT_REVISIT/DONE/' +
  'already-DITCHED entry. Then select the ' + COUNT + ' STRONGEST and most DIVERSE ideas (spread across phases and scopes, ' +
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
    '4. Capture the change as a unified diff: from ' + REPO + ' run `git diff` and put the FULL output in the diff field.\n\n' +
    'Return compiles/tests_pass honestly. If you cannot make it compile or pass tests, return compiles/tests_pass=false, ' +
    'diff="" and explain why in notes. id MUST equal "' + idea.id + '".',
    { label: 'impl:' + idea.id, phase: 'Implement', schema: IMPL_SCHEMA, isolation: 'worktree' }
  )
))).filter(Boolean)
const ready = built.filter((b) => b.compiles && b.tests_pass && b.diff && b.diff.trim().length > 0)
log('Implement: ' + ready.length + '/' + ideas.length + ' compiled + passed tests: ' + ready.map((b) => b.id).join(', '))

// ============================================================================
// SERIAL from here: one sim at a time on a quiet machine (noisy-neighbour).
phase('Measure')
const idById = Object.fromEntries(ideas.map((i) => [i.id, i]))

// Baseline measured ONCE on the current (unpatched) tree.
const baseline = await agent(
  'Establish the perf baseline for an A/B comparison. Work directly in ' + SIM + ' (NO worktree). The working tree already ' +
  'contains the harness; do not modify it.\n\n' +
  '1. mkdir -p ' + ART + '\n' +
  '2. Build: ' + BUILD + '\n' +
  '3. Full baseline: ' + RUN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/base.full.json\n' +
  '4. Triage baseline: ' + RUN + ' --seeds ' + TRIAGE_SEEDS + ' --pop ' + TRIAGE_POP + ' ' + WORLD + ' --out ' + ART + '/base.triage.json\n\n' +
  'Return ok=true and the printed wall mean-of-p99s (ms) for the full run in notes. This run owns the machine; nothing else is running.',
  { label: 'measure:baseline', phase: 'Measure', schema: { type: 'object', properties: { ok: { type: 'boolean' }, wall_p99_ms: { type: 'number' }, notes: { type: 'string' } }, required: ['ok', 'notes'] } }
)

const measured = []
for (const cand of ready) {
  const idea = idById[cand.id] || {}
  const phaseName = idea.target_phase || 'perception'
  const m = await agent(
    'Measure ONE candidate optimization against the pinned baseline. SERIAL — you own the machine; never background a run.\n\n' +
    'Candidate id: ' + cand.id + ' (targets phase: ' + phaseName + ')\n' +
    'The unified diff to apply is between the <DIFF> markers:\n<DIFF>\n' + cand.diff + '\n</DIFF>\n\n' +
    'Protocol (run from ' + REPO + '):\n' +
    '1. Write the diff to ' + ART + '/' + cand.id + '.patch. `git apply --check` it; if it does not apply cleanly, return verdict=ERROR with the reason and STOP (do not modify the tree).\n' +
    '2. `git apply ' + ART + '/' + cand.id + '.patch`\n' +
    '3. Build: (cd ' + SIM + ' && ' + BUILD + '). If it fails to build, `git apply -R` the patch and return verdict=ERROR.\n' +
    '4. TRIAGE: (cd ' + SIM + ' && ' + RUN + ' --seeds ' + TRIAGE_SEEDS + ' --pop ' + TRIAGE_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.triage.json)\n' +
    '   then verdict: ' + RUN + ' --verdict --baseline ' + ART + '/base.triage.json --candidate ' + ART + '/' + cand.id + '.triage.json --phase ' + phaseName + '\n' +
    '   The VERDICT= line prints dPhaseP99/dWallP99/noises in microseconds. If triage VERDICT=Ditch AND dWallP99 is not clearly negative ' +
    '(i.e. no hint of improvement), set triage_only=true and use the triage verdict — skip the full run to save time.\n' +
    '5. Otherwise ESCALATE: (cd ' + SIM + ' && ' + RUN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.full.json)\n' +
    '   then: ' + RUN + ' --verdict --baseline ' + ART + '/base.full.json --candidate ' + ART + '/' + cand.id + '.full.json --phase ' + phaseName + '\n' +
    '6. ALWAYS revert: `git apply -R ' + ART + '/' + cand.id + '.patch` so the tree is clean for the next candidate. Confirm with `git status`.\n\n' +
    'Convert microseconds to ms (÷1000). Return the verdict (KEEP/DEFER/DITCH from the VERDICT= line, the escalated one if you ran it), ' +
    'dwall_p99_ms, dphase_ms, phase_noise_ms, wall_noise_ms, triage_only, and notes. id MUST equal "' + cand.id + '".',
    { label: 'measure:' + cand.id, phase: 'Measure', schema: MEASURE_SCHEMA }
  )
  if (m) measured.push({ ...m, scope: idea.scope, title: idea.title, tradeoffs: idea.tradeoffs, target_phase: m.target_phase || phaseName })
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
    'Protocol (from ' + REPO + '):\n' +
    '1. Apply each patch in turn with `git apply` (check each first). If any conflicts with an already-applied one, skip it and note which.\n' +
    '2. Build: (cd ' + SIM + ' && ' + BUILD + ').\n' +
    '3. Full union run: (cd ' + SIM + ' && ' + RUN + ' --seeds ' + SEEDS + ' --pop ' + FULL_POP + ' ' + WORLD + ' --out ' + ART + '/bundle.full.json)\n' +
    '4. Verdict against the dominant phase (the one with the largest individual phase win among the stacked defers): ' +
    RUN + ' --verdict --baseline ' + ART + '/base.full.json --candidate ' + ART + '/bundle.full.json --phase <dominant_phase>\n' +
    '5. Reverse every applied patch (`git apply -R`) and confirm `git status` is clean.\n\n' +
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
