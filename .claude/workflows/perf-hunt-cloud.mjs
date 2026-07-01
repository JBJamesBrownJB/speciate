export const meta = {
  name: 'perf-hunt-cloud',
  description: 'Cloud TRIAGE hunt: discover Speciate perf ideas, measure them at function/ECS-system granularity + a growth-rate ramp to ~10k, and log the promising ones as PRIME CANDIDATES in the ledger for a full 1M hunt on the home rig. Never merges.',
  whenToUse: 'On a shared/cheap runtime (not a quiet many-core rig). Discovers + triages ideas cheaply and hands winners to /perf-hunt. Pass a number = how many ideas to hunt.',
  phases: [
    { title: 'Baseline', detail: 'build latency_lab; profile per-phase at ~10k + fit the baseline growth exponent (1k→10k sweep)' },
    { title: 'Recall', detail: 'read the ledger (skip DO_NOT_REVISIT/DONE, surface retest) + checklist; brief hunters on the fattest live phase' },
    { title: 'Ideate', detail: 'parallel hunter fleet proposes ideas tagged with a target_phase and (when apt) a criterion bench_target; synthesizer dedupes vs ledger' },
    { title: 'Implement', detail: 'each idea built in an isolated worktree, gated on cargo test -> unified diff' },
    { title: 'Micro-measure', detail: 'per candidate: function-bench delta + per-phase A/B at 10k + growth-rate ramp (Δexponent). Triage signal only — no bank, no escalate' },
    { title: 'Log', detail: 'append promising candidates to the ledger as CANDIDATE+retest for the home rig; stash diffs; write cloud-last-run.md. Never merges' },
  ],
}

// ---- config -----------------------------------------------------------------
let cfg = args
if (typeof cfg === 'string') {
  try { cfg = JSON.parse(cfg) } catch (e) { const n = parseInt(cfg, 10); cfg = Number.isFinite(n) ? { count: n } : {} }
}
if (typeof cfg === 'number') cfg = { count: cfg }
if (!cfg || typeof cfg !== 'object') cfg = {}
const COUNT = cfg.count || 5

// Linux-native + relative to the repo root (agents run from the session cwd = repo
// root). No hardcoded drive letters, no .exe — this is the cloud sibling of the
// Windows-hosted /perf-hunt, deliberately kept separate so that one stays pristine.
const REPO = cfg.repo || '.'
const SIM = REPO + '/apps/simulation'
const LEDGER = REPO + '/docs/scale/perf-hunt/ledger.jsonl'
const CAND_DIR = REPO + '/docs/scale/perf-hunt/candidates'
const REPORT = REPO + '/docs/scale/perf-hunt/cloud-last-run.md'
const SHARED_TARGET = SIM + '/target'
const ART = SHARED_TARGET + '/perf-hunt' // inside target/ => gitignored
const BUILD = 'cargo build --release --features dev-tools --bin latency_lab --target-dir ' + SHARED_TARGET
const RELEASE_BIN = './' + SHARED_TARGET + '/release/latency_lab' // freshly-built candidate
const BASELINE_BIN = './' + ART + '/latency_lab_baseline'          // stashed clean baseline

// Cloud fidelity: tiny pops that fit a 4-core VM. The growth EXPONENT across the
// ramp — not absolute wall time — is the durable signal, since a fitted slope is
// far more robust to a noisy shared machine than a single-point delta.
const MICRO_POP = cfg.microPop || 10000
const MICRO_SEEDS = cfg.microSeeds || '11,42,99'
const SWEEP_FROM = cfg.sweepFrom || 1000
const SWEEP_TO = cfg.sweepTo || 10000
const SWEEP_STEP = cfg.sweepStep || 2250 // → 1000,3250,5500,7750,10000 (5 points)
const WORLD = '--realistic-dna --half-x 5000 --half-y 5000'
const PHASES = ['perception', 'steering', 'movement', 'grid_rebuild', 'l1_aggregation', 'behavior']

const SCOPE_NOTE =
  'Scope: engine micro-opts (behavior-preserving), architectural/risky (schedule overlap, fork-join removal, SoA ' +
  'splits, parallelism), and Golden-Zone biological (perf that IS a gameplay change). EVERY biological idea MUST ' +
  'state its behavior tradeoff or it is rejected. Prefer ideas whose effect shows at SMALL scale (≤10k) or as a ' +
  'GROWTH-RATE change — this cloud hunt cannot see 1M-only tail effects.'

const TRIAGE_NOTE =
  'THIS IS A CLOUD TRIAGE HUNT on a shared VM: it does NOT bank wins or merge anything. It measures each idea cheaply ' +
  '(function benches, per-phase A/B at ' + MICRO_POP + ', and a 1k→10k growth-rate ramp) purely to decide whether the idea ' +
  'is a PRIME CANDIDATE worth a full 1M×5-seed run on the quiet home rig. A candidate is "prime" if its targeted-phase or ' +
  'function delta is clearly negative beyond small-pop noise, OR it lowers the growth exponent b (beats the scaling class).'

// ---- schemas ----------------------------------------------------------------
const IDEA_FIELDS = {
  id: { type: 'string', description: 'stable kebab slug, unique' },
  title: { type: 'string' },
  hypothesis: { type: 'string', description: 'why this should cut time, mechanistically' },
  scope: { type: 'string', enum: ['engine', 'architectural', 'biological'] },
  target_phase: { type: 'string', enum: PHASES },
  bench_target: { type: 'string', description: 'OPTIONAL criterion filter this idea can be measured by, e.g. "perception::first_k_neighbors" or "vector_ops"; empty if none applies' },
  files: { type: 'string', description: 'file:line anchors the change touches' },
  sketch: { type: 'string', description: 'concrete implementation sketch a Rust dev can follow' },
  tradeoffs: { type: 'string', description: 'cost/consequence; REQUIRED non-empty for biological' },
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
    notes: { type: 'string' },
  },
  required: ['id', 'compiles', 'tests_pass', 'diff', 'notes'],
}
const MEASURE_SCHEMA = {
  type: 'object',
  properties: {
    id: { type: 'string' },
    target_phase: { type: 'string' },
    phase_verdict: { type: 'string', enum: ['Keep', 'Defer', 'Ditch', 'Error'], description: 'the latency_lab --verdict result at cloud pop (triage only, NOT a bank)' },
    dwall_p99_ms: { type: 'number', description: 'Δ wall p99 at cloud pop, ms (negative = faster)' },
    dphase_ms: { type: 'number', description: 'Δ targeted-phase median at cloud pop, ms (negative = faster)' },
    bench_delta_pct: { type: 'number', description: 'criterion delta % if a bench_target was measured, else 0' },
    growth_base: { type: 'number', description: 'fitted growth exponent b of the BASELINE sweep' },
    growth_cand: { type: 'number', description: 'fitted growth exponent b of the CANDIDATE sweep' },
    is_prime: { type: 'boolean', description: 'true if this is a PRIME candidate worth a full home-rig run' },
    notes: { type: 'string' },
  },
  required: ['id', 'phase_verdict', 'is_prime', 'notes'],
}

// ============================================================================
phase('Baseline')
const baseline = await agent(
  'Build and PROFILE the clean baseline for a CLOUD perf-triage run. Work from the repository root (run ' +
  '`git rev-parse --show-toplevel` if unsure) in ' + SIM + ' (NO worktree; do not modify source).\n\n' +
  '1. mkdir -p ' + ART + '\n' +
  '2. CLEAN-TREE GUARD: run `git diff --quiet -- apps/simulation/src`. If it exits non-zero, source is dirty — return ok=false and STOP.\n' +
  '3. Build: (cd ' + SIM + ' && ' + BUILD + ')\n' +
  '4. Stash the clean binary so candidates reuse it without rebuilding: cp ' + RELEASE_BIN + ' ' + BASELINE_BIN + '\n' +
  '5. PROFILE per-phase at cloud pop: ' + BASELINE_BIN + ' --seeds ' + MICRO_SEEDS + ' --pop ' + MICRO_POP + ' ' + WORLD + '\n' +
  '   It prints a "phase <name> p99=<us> ..." line each. Convert µs→ms (÷1000).\n' +
  '6. GROWTH RAMP: ' + BASELINE_BIN + ' --sweep --sweep-from ' + SWEEP_FROM + ' --sweep-to ' + SWEEP_TO + ' --sweep-step ' + SWEEP_STEP + ' ' + WORLD + ' --out ' + ART + '/baseline.sweep.json\n' +
  '   It prints "growth_exponent=<b>" and writes {points, growthExponent} JSON. Read that exponent.\n' +
  'Return ok=true, wall_p99_ms, phases={' + PHASES.join(',') + '} in ms, growth_exponent, and fattest = phase names sorted DESCENDING by p99.',
  { label: 'baseline-profile', phase: 'Baseline', schema: { type: 'object', properties: {
      ok: { type: 'boolean' },
      wall_p99_ms: { type: 'number' },
      phases: { type: 'object', properties: Object.fromEntries(PHASES.map((p) => [p, { type: 'number' }])) },
      growth_exponent: { type: 'number' },
      fattest: { type: 'array', items: { type: 'string' } },
      notes: { type: 'string' },
    }, required: ['ok', 'notes'] } }
)
if (!baseline || !baseline.ok) {
  log('Baseline failed (' + (baseline?.notes || 'agent died') + ') — aborting.')
  return { aborted: 'baseline-failed', notes: baseline?.notes || 'baseline agent returned null' }
}
const liveProfile = baseline.phases
  ? 'LIVE CLOUD BASELINE at ' + MICRO_POP + ' creatures (authoritative for THIS run): wall p99 ' +
    (typeof baseline.wall_p99_ms === 'number' ? baseline.wall_p99_ms.toFixed(2) : '?') + ' ms; per-phase p99 (ms): ' +
    PHASES.map((k) => k + ' ' + (typeof baseline.phases[k] === 'number' ? baseline.phases[k].toFixed(2) : '?')).join(', ') +
    (typeof baseline.growth_exponent === 'number' ? '. Baseline GROWTH EXPONENT b=' + baseline.growth_exponent.toFixed(3) + ' (1 = O(n), 2 = O(n²))' : '') +
    (baseline.fattest && baseline.fattest.length ? '. FATTEST→leanest: ' + baseline.fattest.join(' > ') : '') + '.\n'
  : 'LIVE CLOUD BASELINE: wall p99 ' + (baseline.wall_p99_ms ?? '?') + ' ms.\n'
log('Baseline: wall p99 ' + (baseline.wall_p99_ms ?? '?') + ' ms; growth b=' + (baseline.growth_exponent?.toFixed?.(3) ?? '?') + '; fattest ' + (baseline.fattest && baseline.fattest[0] || '?'))

// ============================================================================
phase('Recall')
const brief = await agent(
  'You are briefing a fleet of performance-optimization hunters for the Speciate Rust/Bevy ECS engine, for a CLOUD ' +
  'TRIAGE run.\n\n' + liveProfile + '\n' + TRIAGE_NOTE + '\n\nTARGET the fattest LIVE phases above. READ, in order:\n' +
  '1. ' + LEDGER + ' (JSONL: every idea already tried + verdict. DO_NOT_REVISIT and DONE are HARD exclusions. An entry ' +
  'with a "retest" field is RE-ELIGIBLE — surface those as priority.)\n' +
  '2. ' + REPO + '/docs/scale/optimization-checklist.md (Pass bar, attack order, ditched list, next levers)\n' +
  '3. Skim the hot systems: apps/simulation/src/simulation/{perception,steering,movement}/ and spatial/.\n\n' +
  'Produce a concise BRIEF (~300 words) covering: (a) the fattest phases RIGHT NOW, (b) ideas already tried they MUST NOT ' +
  're-propose, (c) the most promising untried levers, (d) reminder that this run rewards SMALL-SCALE-visible or ' +
  'GROWTH-RATE-changing ideas because it measures at ≤10k, not 1M.',
  { label: 'recall-ledger', phase: 'Recall' }
)

// ============================================================================
phase('Ideate')
const ANGLES = [
  { key: 'micro', lens: 'engine micro-optimizations: cache layout, allocation, branch elimination, SoA access, reducing per-entity hot-loop work — behavior-preserving; many map to a criterion bench_target (vector_ops, perception::first_k_neighbors, export_sort)' },
  { key: 'arch', lens: 'architectural changes that alter the GROWTH RATE: removing an O(n²) neighbour scan, better spatial pruning, killing fork-join barriers, parallelizing a serial phase — these show as a lower growth exponent even at small scale' },
  { key: 'bio', lens: 'Golden-Zone biological levers where the perf win IS a gameplay mechanic: hunger-gated perception, giants ignoring tiny entities, satiated-predator rest — each MUST carry a behavior tradeoff' },
]
const perHunter = Math.max(2, Math.ceil(COUNT / 2))
const proposals = (await parallel(ANGLES.map((a) => () =>
  agent(
    'You are a Speciate performance hunter focused on ' + a.lens + '.\n\nBRIEF:\n' + brief + '\n\n' + SCOPE_NOTE + '\n\n' +
    'Propose up to ' + perHunter + ' DISTINCT, concrete ideas in your lens. Each needs a real implementation sketch with ' +
    'file:line anchors (read the actual code). Set bench_target when the idea maps to an existing criterion benchmark, else ' +
    'leave it empty. Do NOT propose anything on the ledger DO_NOT_REVISIT/DONE list. Be honest about impact and tradeoffs.',
    { label: 'hunt:' + a.key, phase: 'Ideate', schema: IDEAS_SCHEMA }
  )
))).filter(Boolean).flatMap((r) => r.ideas || [])

const synth = await agent(
  'You are the lead engineer triaging ' + proposals.length + ' proposed perf ideas for a CLOUD triage run.\n\n' +
  'PROPOSALS (JSON):\n' + JSON.stringify(proposals) + '\n\n' +
  'Cross-check every proposal against the ledger ' + LEDGER + ' — DROP anything duplicating a DO_NOT_REVISIT/DONE/already-' +
  'DITCHED entry, EXCEPT entries carrying a "retest" field (prefer those). Then select the ' + COUNT + ' STRONGEST and most ' +
  'DIVERSE ideas (spread across phases/scopes), PREFERRING ideas measurable at small scale or as a growth-rate change. ' +
  'Merge near-duplicates; keep the best sketch. Return exactly ' + COUNT + ' ideas (or fewer), each with a unique id.',
  { label: 'ideate:synthesize', phase: 'Ideate', schema: IDEAS_SCHEMA }
)
const ideas = (synth?.ideas || []).slice(0, COUNT)
log('Ideate: ' + proposals.length + ' proposed -> ' + ideas.length + ' selected: ' + ideas.map((i) => i.id).join(', '))

// ============================================================================
phase('Implement')
log('Pre-warming shared build cache...')
const warm = await agent(
  'Pre-warm the shared Rust build cache. From the repo root run: (cd ' + SIM + ' && ' + BUILD + ')\nReport ok=true if it ' +
  'finishes (wall time in notes), else ok=false with the error.',
  { label: 'prewarm-build', phase: 'Implement', schema: { type: 'object', properties: { ok: { type: 'boolean' }, notes: { type: 'string' } }, required: ['ok', 'notes'] } }
)
if (!warm || !warm.ok) {
  log('Pre-warm failed (' + (warm?.notes || 'agent died') + ') — aborting before fan-out.')
  return { aborted: 'prewarm-failed', notes: warm?.notes || 'prewarm agent returned null' }
}

const built = (await parallel(ideas.map((idea) => () =>
  agent(
    'Implement ONE performance optimization in the Speciate engine, in your isolated worktree.\n\n' +
    'IDEA: ' + JSON.stringify(idea) + '\n\n' +
    'Steps:\n' +
    '1. Read the files in the sketch and implement the MINIMAL change that realizes the hypothesis. Behavior-preserving ' +
    'unless scope is biological (then the behavior change must match the stated design).\n' +
    '2. Do NOT touch apps/simulation/src/bench_lab/ or src/bin/latency_lab.rs (the harness must stay constant).\n' +
    '3. From ' + SIM + ', run `cargo test --target-dir ' + SHARED_TARGET + '` (default features). It MUST pass — fix your ' +
    'change until it does. A "Blocking waiting for file lock on build directory" message is NORMAL — just wait.\n' +
    '4. Capture the change as a unified diff: from the repo root run `git diff` and put the FULL output in the diff field.\n' +
    '5. CLEANUP — REQUIRED: once the diff is captured, restore this worktree to pristine so it auto-removes: from the repo ' +
    'root run `git reset --hard && git clean -fd`.\n\n' +
    'Return compiles/tests_pass honestly. If you cannot make it compile or pass tests, return false, diff="" and explain ' +
    'in notes (still run step 5). id MUST equal "' + idea.id + '".',
    { label: 'impl:' + idea.id, phase: 'Implement', schema: IMPL_SCHEMA, isolation: 'worktree' }
  )
))).filter(Boolean)
const ready = built.filter((b) => b.compiles && b.tests_pass && b.diff && b.diff.trim().length > 0)
log('Implement: ' + ready.length + '/' + ideas.length + ' compiled + passed tests: ' + ready.map((b) => b.id).join(', '))

// ============================================================================
// SERIAL from here: one sim at a time (even on a shared VM, back-to-back A/B
// cancels drift). But this is TRIAGE — cheap pops, no escalate, no bank.
phase('Micro-measure')
const idById = Object.fromEntries(ideas.map((i) => [i.id, i]))
const measured = []
for (const cand of ready) {
  const idea = idById[cand.id] || {}
  const phaseName = idea.target_phase || 'perception'
  const benchTarget = (idea.bench_target || '').trim()
  const m = await agent(
    'MICRO-MEASURE one candidate for a CLOUD triage hunt. SERIAL — never background a run. This is triage: cheap pops, ' +
    'NO escalation to 1M, NO banking. Baseline binary ' + BASELINE_BIN + ' is prebuilt — run it as-is, never rebuild it.\n\n' +
    'Candidate id: ' + cand.id + ' (targets phase: ' + phaseName + ')\n' +
    (benchTarget ? 'Criterion bench_target: ' + benchTarget + '\n' : '') +
    'Baseline growth exponent b=' + (typeof baseline.growth_exponent === 'number' ? baseline.growth_exponent.toFixed(3) : '?') + '\n' +
    'Diff between the <DIFF> markers:\n<DIFF>\n' + cand.diff + '\n</DIFF>\n\n' +
    'Protocol (from the repo root):\n' +
    '1. CLEAN-TREE GUARD: `git diff --quiet -- apps/simulation/src`. If dirty return phase_verdict="Error", is_prime=false, STOP.\n' +
    '2. Write the diff to ' + ART + '/' + cand.id + '.patch; `git apply --check` it; if it fails, return phase_verdict="Error", is_prime=false, STOP.\n' +
    '3. `git apply ' + ART + '/' + cand.id + '.patch`; build candidate: (cd ' + SIM + ' && ' + BUILD + '). If build fails, `git apply -R` and return phase_verdict="Error", is_prime=false. Candidate binary is now ' + RELEASE_BIN + '.\n' +
    (benchTarget
      ? '4. FUNCTION BENCH (optional signal): run the criterion filter on baseline vs candidate. Since the baseline binary is a separate build, measure the candidate now with `(cd ' + SIM + ' && cargo bench --bench simulation_bench -- ' + benchTarget + ')` and read criterion\'s own "change: [-x% .. +y%]" line vs its stored baseline; record the midpoint as bench_delta_pct (negative = faster). If criterion has no stored baseline yet, run it twice and note that in notes; set bench_delta_pct=0 if unavailable.\n'
      : '4. (No bench_target for this idea — skip the function bench, set bench_delta_pct=0.)\n') +
    '5. PER-PHASE A/B at cloud pop, back-to-back BASELINE FIRST then candidate:\n' +
    '   ' + BASELINE_BIN + ' --seeds ' + MICRO_SEEDS + ' --pop ' + MICRO_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.base.json\n' +
    '   ' + RELEASE_BIN + ' --seeds ' + MICRO_SEEDS + ' --pop ' + MICRO_POP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.cand.json\n' +
    '   verdict: ' + RELEASE_BIN + ' --verdict --baseline ' + ART + '/' + cand.id + '.base.json --candidate ' + ART + '/' + cand.id + '.cand.json --phase ' + phaseName + '\n' +
    '   Parse phase_verdict (Keep/Defer/Ditch) and dPhaseMedian/dWallMedian (µs → ms ÷1000).\n' +
    '6. GROWTH RAMP for the candidate: ' + RELEASE_BIN + ' --sweep --sweep-from ' + SWEEP_FROM + ' --sweep-to ' + SWEEP_TO + ' --sweep-step ' + SWEEP_STEP + ' ' + WORLD + ' --out ' + ART + '/' + cand.id + '.cand.sweep.json\n' +
    '   Read growthExponent from that JSON → growth_cand. growth_base=' + (typeof baseline.growth_exponent === 'number' ? baseline.growth_exponent.toFixed(3) : 'the baseline value') + '.\n' +
    '7. REVERT + GUARD: `git apply -R ' + ART + '/' + cand.id + '.patch`, then `git diff --quiet -- apps/simulation/src` MUST pass. If not, note "revert FAILED — tree dirty" so the next candidate aborts.\n\n' +
    'PRIME DECISION (is_prime=true) if ANY holds: phase_verdict is Keep or Defer; OR dphase_ms clearly negative beyond noise; ' +
    'OR bench_delta_pct clearly negative; OR growth_cand meaningfully below growth_base (a scaling-class win). Otherwise is_prime=false. ' +
    'Return all fields. id MUST equal "' + cand.id + '".',
    { label: 'measure:' + cand.id, phase: 'Micro-measure', schema: MEASURE_SCHEMA }
  )
  if (m) measured.push({ ...m, scope: idea.scope, title: idea.title, target_phase: m.target_phase || phaseName })
}
const primes = measured.filter((m) => m.is_prime && m.phase_verdict !== 'Error')
log('Micro-measure: ' + measured.length + ' measured -> ' + primes.length + ' PRIME candidates for the home rig')

// ============================================================================
phase('Log')
const today = 'run `date +%Y-%m-%d`'
const logInput = {
  cloud_config: { count: COUNT, micro_pop: MICRO_POP, seeds: MICRO_SEEDS, sweep: SWEEP_FROM + '..' + SWEEP_TO + ' step ' + SWEEP_STEP },
  baseline: { wall_p99_ms: baseline.wall_p99_ms, growth_exponent: baseline.growth_exponent, fattest: baseline.fattest },
  measured,
  primes,
}
const report = await agent(
  'Finalize a CLOUD perf-triage run: LOG prime candidates to the shared ledger for the home rig, and write a human report. ' +
  'This run NEVER merges engine changes and NEVER banks a win — it only hands prioritized candidates to the full /perf-hunt.\n\n' +
  'RESULTS (JSON):\n' + JSON.stringify(logInput) + '\n\n' +
  'PART A — LEDGER (append-only): get today via ' + today + '. For EACH prime candidate, append ONE JSONL line to ' + LEDGER + '. ' +
  'Use exactly these fields and this key order (this schema is guarded by .claude/workflows/lib/cloud-ledger.mjs):\n' +
  '  {"id","date","title","scope","target_phase","verdict","dwall_p99_ms","dphase_ms","notes","origin","retest"}\n' +
  'Set verdict="CANDIDATE", origin="cloud-triage". notes = the cloud signal (pop=' + MICRO_POP + ', Δphase, Δwall, bench %, growth b base→cand). ' +
  'retest = "cloud-triage <date>: <one-line why it is promising incl. growth b base→cand>; needs full 1M validation on the home rig." ' +
  'This retest field is what makes the full /perf-hunt surface it as a PRIORITY re-test. Do NOT rewrite existing ledger lines; append only. ' +
  'Do NOT log non-prime candidates to the ledger.\n\n' +
  'PART B — STASH DIFFS: for each prime candidate, ensure its patch is saved to ' + CAND_DIR + '/<id>.diff (copy from ' + ART + '/<id>.patch).\n\n' +
  'PART C — REPORT: write a skimmable markdown report to ' + REPORT + ' with a table: candidate | scope | target phase | ' +
  'phase_verdict | Δphase (ms) | Δwall (ms) | bench Δ% | growth b (base→cand) | PRIME? . List primes first with a one-line ' +
  '"why prime + what the home rig should confirm" for each. Include a clear banner that these are CLOUD TRIAGE signals on a ' +
  'shared VM — not authoritative — and that the home-rig /perf-hunt will pick up the logged candidates automatically via their retest field.\n\n' +
  'Return a tight plain-text executive summary (the table + the prime shortlist) as your final message.',
  { label: 'log+report', phase: 'Log' }
)

return {
  ideasConsidered: ideas.length,
  implemented: ready.length,
  measured: measured.length,
  primes: primes.map((p) => ({ id: p.id, phase_verdict: p.phase_verdict, dphase_ms: p.dphase_ms, growth_base: p.growth_base, growth_cand: p.growth_cand })),
  reportPath: REPORT,
  note: 'Cloud triage only — no wins banked, nothing merged. Prime candidates logged to the ledger for the home-rig /perf-hunt.',
  summary: report,
}
