// Canonical schema for the ledger rows the cloud triage hunt appends to
// docs/scale/perf-hunt/ledger.jsonl. This module is the SOURCE OF TRUTH for the
// row shape, and it is ON the execution path: the perf-hunt-cloud "Log" phase
// appends rows by running ledger-cli.mjs (which calls these builders), then
// gates the file with lintLedger — rows are never hand-written.
//
// WHY a cloud row is a `CANDIDATE` with `retest`: the existing full-fidelity
// /perf-hunt (perf-hunt.mjs) surfaces any ledger entry carrying a `retest` field
// as a PRIORITY re-test and PREFERS it over fresh ideas. So a cloud triage hit
// hands off to the home rig with zero changes to the full workflow.

/** Fields every ledger row must carry (base schema shared with the full hunt). */
export const REQUIRED_FIELDS = [
  'id', 'date', 'title', 'scope', 'target_phase',
  'verdict', 'dwall_p99_ms', 'dphase_ms', 'notes',
];

const SCOPES = ['engine', 'architectural', 'biological', 'none'];
const PHASES = ['perception', 'steering', 'movement', 'grid_rebuild', 'l1_aggregation', 'behavior', 'none'];

/**
 * Build one JSONL ledger line (no trailing newline) for a cloud-triage prime
 * candidate. Defaults verdict→CANDIDATE and origin→cloud-triage, and synthesizes
 * a `retest` note from the cloud signal when one isn't supplied, so the home-rig
 * hunt auto-prioritizes it. Throws on missing/invalid inputs rather than emitting
 * a malformed row the downstream reader would choke on.
 *
 * @param {object} f
 * @param {string} f.id            stable kebab slug, unique
 * @param {string} f.date          YYYY-MM-DD
 * @param {string} f.title         one-line description
 * @param {string} f.scope         engine|architectural|biological
 * @param {string} f.target_phase  perception|steering|movement|grid_rebuild|l1_aggregation|behavior
 * @param {number} f.dwall_p99_ms  Δ wall p99 at the cloud test pop (negative = faster)
 * @param {number} f.dphase_ms     Δ targeted-phase (negative = faster)
 * @param {string} f.notes         cloud-triage context (pop, deltas, why it's a candidate)
 * @param {number} [f.growth_base] fitted exponent b of the baseline sweep
 * @param {number} [f.growth_cand] fitted exponent b of the candidate sweep
 * @param {string} [f.verdict]     defaults to 'CANDIDATE'
 * @param {string} [f.origin]      defaults to 'cloud-triage'
 * @param {string} [f.retest]      overrides the synthesized re-eligibility note
 * @returns {string} a single-line JSON string
 */
function assertBaseFields(f, who) {
  if (!f || typeof f !== 'object') throw new Error(`${who}: fields object required`);
  if (!f.id || typeof f.id !== 'string') throw new Error(`${who}: id (string) required`);
  if (!f.title || typeof f.title !== 'string') throw new Error(`${who}: title (string) required`);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(f.date || '')) throw new Error(`${who}: date must be YYYY-MM-DD`);
  if (!SCOPES.includes(f.scope)) throw new Error(`${who}: scope must be one of ${SCOPES.join('|')}`);
  if (!PHASES.includes(f.target_phase)) throw new Error(`${who}: target_phase must be one of ${PHASES.join('|')}`);
  if (typeof f.dwall_p99_ms !== 'number' || Number.isNaN(f.dwall_p99_ms)) throw new Error(`${who}: dwall_p99_ms (number) required`);
  if (typeof f.dphase_ms !== 'number' || Number.isNaN(f.dphase_ms)) throw new Error(`${who}: dphase_ms (number) required`);
  if (!f.notes || typeof f.notes !== 'string') throw new Error(`${who}: notes (string) required`);
}

/** Single owner of the shared row shape and key order (fixed order keeps the
 *  JSONL diff-friendly). Candidate rows add a trailing `retest` on top. */
function baseRow(f, defaultVerdict) {
  return {
    id: f.id,
    date: f.date,
    title: f.title,
    scope: f.scope,
    target_phase: f.target_phase,
    verdict: f.verdict || defaultVerdict,
    dwall_p99_ms: f.dwall_p99_ms,
    dphase_ms: f.dphase_ms,
    notes: f.notes,
    origin: f.origin || 'cloud-triage',
  };
}

export function buildCandidateLine(f) {
  assertBaseFields(f, 'buildCandidateLine');

  const growth = (typeof f.growth_base === 'number' && typeof f.growth_cand === 'number')
    ? `, growth b ${f.growth_base.toFixed(2)}→${f.growth_cand.toFixed(2)}`
    : '';
  const retest = f.retest || `cloud-triage ${f.date}: Δphase ${f.dphase_ms.toFixed(2)}ms / Δwall ${f.dwall_p99_ms.toFixed(2)}ms at cloud pop${growth}; needs full 1M validation on the home rig.`;

  return JSON.stringify({ ...baseRow(f, 'CANDIDATE'), retest });
}

/**
 * Build one JSONL ledger line for a cloud-triage idea that was TRIED but judged
 * NOT prime. This is a SOFT exclusion so future cloud hunts don't waste cycles
 * re-implementing and re-measuring the same losing idea — but deliberately NOT a
 * home-rig kill: the row carries verdict `CLOUD_TRIED` and, crucially, NO `retest`
 * field, so the full /perf-hunt neither prioritizes it nor treats it as
 * DO_NOT_REVISIT. A noisy ≤10k shared-VM measurement must never permanently bury
 * an idea whose payoff only appears at 1M. Same validation and key order as
 * {@link buildCandidateLine}, minus `retest`.
 *
 * @param {object} f same base fields as buildCandidateLine (id, date, title,
 *   scope, target_phase, dwall_p99_ms, dphase_ms, notes)
 * @param {string} [f.verdict] defaults to 'CLOUD_TRIED'
 * @param {string} [f.origin]  defaults to 'cloud-triage'
 * @returns {string} a single-line JSON string with no `retest` key
 */
export function buildTriedLine(f) {
  assertBaseFields(f, 'buildTriedLine');
  return JSON.stringify(baseRow(f, 'CLOUD_TRIED'));
}

/** CLI dispatch: which guarded builder produces a row of the given kind. */
export function buildRow(kind, fields) {
  if (kind === 'candidate') return buildCandidateLine(fields);
  if (kind === 'tried') return buildTriedLine(fields);
  throw new Error(`buildRow: kind must be candidate|tried, got "${kind}"`);
}

/**
 * Parse a ledger line into an object with the base fields guaranteed present.
 * Backward-compatible: older rows (pre-cloud, missing `origin`/`retest`) parse
 * cleanly, with `origin` defaulting to 'home-rig' and `retest` to null — so a
 * reader can treat every row uniformly regardless of when it was written.
 */
export function parseLedgerLine(line) {
  const obj = JSON.parse(line);
  for (const k of REQUIRED_FIELDS) {
    if (!(k in obj)) throw new Error(`parseLedgerLine: row missing required field "${k}"`);
  }
  return { origin: 'home-rig', retest: null, ...obj };
}

/**
 * Mechanically validate a whole ledger file's text — the gate the cloud Log
 * phase runs AFTER appending, so a malformed hand-off can never silently land
 * in the append-only file. Beyond per-row parseability it pins the two
 * invariants the verdicts encode:
 *  - CLOUD_TRIED must NOT carry `retest` (a stray retest would turn a soft
 *    exclusion into a home-rig PRIORITY re-test of a known loser);
 *  - CANDIDATE must carry `retest` (that field is what makes the home-rig
 *    /perf-hunt surface it at all).
 *
 * @param {string} text full ledger contents
 * @returns {{ok: boolean, errors: Array<{line: number, message: string}>}}
 */
export function lintLedger(text) {
  const errors = [];
  text.split('\n').forEach((line, i) => {
    if (!line.trim()) return; // blank/trailing lines are fine
    const lineNo = i + 1;
    let row;
    try {
      row = parseLedgerLine(line);
    } catch (e) {
      errors.push({ line: lineNo, message: e.message });
      return;
    }
    if (row.verdict === 'CLOUD_TRIED' && row.retest !== null) {
      errors.push({ line: lineNo, message: `CLOUD_TRIED row "${row.id}" must not carry retest (soft exclusion would become a priority re-test)` });
    }
    if (row.verdict === 'CANDIDATE' && !row.retest) {
      errors.push({ line: lineNo, message: `CANDIDATE row "${row.id}" must carry retest (the home rig surfaces candidates via that field)` });
    }
  });
  return { ok: errors.length === 0, errors };
}
