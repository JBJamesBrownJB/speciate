// Run: node --test .claude/workflows/lib/cloud-normalize.test.mjs
//
// The perf-hunt-cloud workflow runs in a sandbox that cannot import modules,
// so its post-measure normalization (missing-measurement guard, true Δwall p99
// from the A/B absolutes, growth-aware 1M projection) is inline JS. This test
// EXTRACTS that inline block from the workflow source (between the
// NORMALIZE-INLINE markers) and EXECUTES it, pinning it against the tested
// formula in perf-projection.mjs — the drift the old "keep in sync" comment
// could not prevent.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { projectSavingsTo1M } from './perf-projection.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const workflowSrc = readFileSync(resolve(here, '../perf-hunt-cloud.mjs'), 'utf8');

const match = workflowSrc.match(/\/\/ NORMALIZE-INLINE-START\r?\n([\s\S]*?)\/\/ NORMALIZE-INLINE-END/);

/** Execute the extracted block against fixture data; returns emitted log lines. */
function runNormalize(measured, baseline, { targetPop = 1000000, microPop = 10000 } = {}) {
  const logs = [];
  const fn = new Function(
    'measured', 'baseline', 'TARGET_POP', 'MICRO_POP', 'log',
    '"use strict";\n' + match[1]
  );
  fn(measured, baseline, targetPop, microPop, (s) => logs.push(String(s)));
  return logs;
}

const completeRow = () => ({
  id: 'cache-cell-index-scatter',
  phase_verdict: 'Keep',
  is_prime: true,
  wall_base_ms: 3.126,
  wall_cand_ms: 2.871,
  growth_base: 0.361,
  growth_cand: 0.331,
  dwall_p99_ms: -0.077, // agent-parsed median — must be overwritten from absolutes
  dphase_ms: -0.053,
  notes: 'n',
});

test('the workflow source contains exactly one extractable NORMALIZE-INLINE block', () => {
  assert.ok(match, 'NORMALIZE-INLINE-START/END markers missing from perf-hunt-cloud.mjs');
  assert.equal(workflowSrc.split('NORMALIZE-INLINE-START').length, 2, 'markers must be unique');
});

test('inline projection matches the tested lib formula exactly (no drift)', () => {
  const m = completeRow();
  runNormalize([m], { growth_exponent: 0.361 });

  const lib = projectSavingsTo1M({
    wallBaseMs: 3.126, wallCandMs: 2.871, growthBase: 0.361, growthCand: 0.331,
  });
  assert.equal(m.proj_base_1m_ms, lib.projBase1MMs);
  assert.equal(m.proj_cand_1m_ms, lib.projCand1MMs);
  assert.equal(m.proj_savings_1m_ms, lib.projSavings1MMs);
});

test('inline matches the lib across a spread of inputs (parametrized drift check)', () => {
  const cases = [
    { wall_base_ms: 4.082, wall_cand_ms: 4.004, growth_base: 0.361, growth_cand: 0.559 },
    { wall_base_ms: 4, wall_cand_ms: 4, growth_base: 0.5, growth_cand: 0.5 },
    { wall_base_ms: 10.5, wall_cand_ms: 9.1, growth_base: 1.0, growth_cand: 0.9 },
  ];
  for (const c of cases) {
    const m = { ...completeRow(), ...c };
    runNormalize([m], { growth_exponent: c.growth_base });
    const lib = projectSavingsTo1M({
      wallBaseMs: c.wall_base_ms, wallCandMs: c.wall_cand_ms,
      growthBase: c.growth_base, growthCand: c.growth_cand,
    });
    assert.equal(m.proj_savings_1m_ms, lib.projSavings1MMs, JSON.stringify(c));
  }
});

test('dwall_p99_ms is recomputed from the A/B absolutes (median parse is not trusted)', () => {
  const m = completeRow();
  runNormalize([m], { growth_exponent: 0.361 });
  // 2.871 − 3.126 = −0.255 (true Δ p99), NOT the agent-parsed −0.077 median.
  assert.equal(m.dwall_p99_ms, -0.255);
});

test('a non-Error row missing its measurement numbers is downgraded to Error, never logged as tried', () => {
  const m = { ...completeRow(), wall_cand_ms: undefined };
  const logs = runNormalize([m], { growth_exponent: 0.361 });

  assert.equal(m.phase_verdict, 'Error');
  assert.equal(m.is_prime, false);
  assert.match(m.notes, /MEASUREMENT INCOMPLETE/);
  assert.match(m.notes, /wall_cand_ms/);
  assert.equal(m.proj_savings_1m_ms, null);
  assert.ok(logs.some((l) => l.includes(m.id)), 'projection gap is logged, not silent');
});

test('growth_base falls back to the baseline exponent; both missing downgrades to Error', () => {
  const withFallback = { ...completeRow(), growth_base: undefined };
  runNormalize([withFallback], { growth_exponent: 0.361 });
  assert.equal(withFallback.growth_base, 0.361);
  assert.equal(
    withFallback.proj_savings_1m_ms,
    projectSavingsTo1M({ wallBaseMs: 3.126, wallCandMs: 2.871, growthBase: 0.361, growthCand: 0.331 }).projSavings1MMs
  );

  const noExponent = { ...completeRow(), growth_base: undefined };
  runNormalize([noExponent], {});
  assert.equal(noExponent.phase_verdict, 'Error');
  assert.match(noExponent.notes, /growth_base/);
});

test('an Error row (infra failure) passes through: no downgrade text, null projection, logged', () => {
  const m = { id: 'broken-patch', phase_verdict: 'Error', is_prime: false, notes: 'patch failed' };
  const logs = runNormalize([m], { growth_exponent: 0.361 });
  assert.equal(m.phase_verdict, 'Error');
  assert.equal(m.notes, 'patch failed');
  assert.equal(m.proj_savings_1m_ms, null);
  assert.ok(logs.some((l) => l.includes('broken-patch')));
});
