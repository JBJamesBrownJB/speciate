// Canonical schema for the ledger rows the cloud triage hunt appends to
// docs/scale/perf-hunt/ledger.jsonl. This module is the SOURCE OF TRUTH for the
// row shape: the perf-hunt-cloud workflow's "Log candidates" phase instructs its
// agent to append lines in exactly this format, and node --test guards it here.
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
export function buildCandidateLine(f) {
  if (!f || typeof f !== 'object') throw new Error('buildCandidateLine: fields object required');
  if (!f.id || typeof f.id !== 'string') throw new Error('buildCandidateLine: id (string) required');
  if (!f.title || typeof f.title !== 'string') throw new Error('buildCandidateLine: title (string) required');
  if (!/^\d{4}-\d{2}-\d{2}$/.test(f.date || '')) throw new Error('buildCandidateLine: date must be YYYY-MM-DD');
  if (!SCOPES.includes(f.scope)) throw new Error(`buildCandidateLine: scope must be one of ${SCOPES.join('|')}`);
  if (!PHASES.includes(f.target_phase)) throw new Error(`buildCandidateLine: target_phase must be one of ${PHASES.join('|')}`);
  if (typeof f.dwall_p99_ms !== 'number' || Number.isNaN(f.dwall_p99_ms)) throw new Error('buildCandidateLine: dwall_p99_ms (number) required');
  if (typeof f.dphase_ms !== 'number' || Number.isNaN(f.dphase_ms)) throw new Error('buildCandidateLine: dphase_ms (number) required');
  if (!f.notes || typeof f.notes !== 'string') throw new Error('buildCandidateLine: notes (string) required');

  const growth = (typeof f.growth_base === 'number' && typeof f.growth_cand === 'number')
    ? `, growth b ${f.growth_base.toFixed(2)}→${f.growth_cand.toFixed(2)}`
    : '';
  const retest = f.retest || `cloud-triage ${f.date}: Δphase ${f.dphase_ms.toFixed(2)}ms / Δwall ${f.dwall_p99_ms.toFixed(2)}ms at cloud pop${growth}; needs full 1M validation on the home rig.`;

  // Fixed key order keeps the JSONL diff-friendly and mirrors the full hunt's rows.
  const row = {
    id: f.id,
    date: f.date,
    title: f.title,
    scope: f.scope,
    target_phase: f.target_phase,
    verdict: f.verdict || 'CANDIDATE',
    dwall_p99_ms: f.dwall_p99_ms,
    dphase_ms: f.dphase_ms,
    notes: f.notes,
    origin: f.origin || 'cloud-triage',
    retest,
  };
  return JSON.stringify(row);
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
