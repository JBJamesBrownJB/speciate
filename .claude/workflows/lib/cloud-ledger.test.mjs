// Run: node --test .claude/workflows/lib/cloud-ledger.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buildCandidateLine, parseLedgerLine, REQUIRED_FIELDS } from './cloud-ledger.mjs';

const valid = {
  id: 'perception-range-trim-cloud',
  date: '2026-07-01',
  title: 'Trim perception range via allometry exponent',
  scope: 'engine',
  target_phase: 'perception',
  dwall_p99_ms: -0.42,
  dphase_ms: -1.1,
  notes: 'cloud triage @10k: phase -1.1ms, wall -0.42ms',
};

test('emits every required ledger field plus origin + retest', () => {
  const row = JSON.parse(buildCandidateLine(valid));
  for (const k of REQUIRED_FIELDS) assert.ok(k in row, `missing ${k}`);
  assert.equal(row.verdict, 'CANDIDATE');
  assert.equal(row.origin, 'cloud-triage');
  assert.ok(typeof row.retest === 'string' && row.retest.length > 0, 'retest note present');
});

test('synthesizes a retest note that mentions the growth exponents when given', () => {
  const row = JSON.parse(buildCandidateLine({ ...valid, growth_base: 1.32, growth_cand: 1.08 }));
  assert.match(row.retest, /growth b 1\.32→1\.08/);
  assert.match(row.retest, /home rig/i);
});

test('caller-supplied verdict/origin/retest override the defaults', () => {
  const row = JSON.parse(buildCandidateLine({ ...valid, verdict: 'DEFER', origin: 'x', retest: 'custom' }));
  assert.equal(row.verdict, 'DEFER');
  assert.equal(row.origin, 'x');
  assert.equal(row.retest, 'custom');
});

test('rejects malformed input rather than emitting a bad row', () => {
  assert.throws(() => buildCandidateLine({ ...valid, id: '' }), /id/);
  assert.throws(() => buildCandidateLine({ ...valid, date: '7/1/26' }), /YYYY-MM-DD/);
  assert.throws(() => buildCandidateLine({ ...valid, scope: 'bogus' }), /scope/);
  assert.throws(() => buildCandidateLine({ ...valid, target_phase: 'nope' }), /target_phase/);
  assert.throws(() => buildCandidateLine({ ...valid, dwall_p99_ms: 'x' }), /dwall_p99_ms/);
});

test('output is a single JSONL line (no embedded newline)', () => {
  assert.ok(!buildCandidateLine(valid).includes('\n'), 'must be one line');
});

test('backward-compat: an old row missing origin/retest still parses', () => {
  // A pre-cloud full-hunt row: base fields only, no origin/retest.
  const old = JSON.stringify({
    id: 'native-par-iter', date: '2026-06-20', title: 'kill 1M collect',
    scope: 'architectural', target_phase: 'movement', verdict: 'KEEP',
    dwall_p99_ms: -6.9, dphase_ms: -6.9, notes: 'merged',
  });
  const parsed = parseLedgerLine(old);
  assert.equal(parsed.origin, 'home-rig', 'defaults origin for legacy rows');
  assert.equal(parsed.retest, null, 'defaults retest for legacy rows');
  assert.equal(parsed.verdict, 'KEEP');
});

test('round-trips a freshly built candidate line through the parser', () => {
  const parsed = parseLedgerLine(buildCandidateLine(valid));
  assert.equal(parsed.id, valid.id);
  assert.equal(parsed.origin, 'cloud-triage');
});
