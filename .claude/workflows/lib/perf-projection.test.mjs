// Run: node --test .claude/workflows/lib/perf-projection.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { projectSavingsTo1M } from './perf-projection.mjs';

const near = (a, b, eps = 0.01) => Math.abs(a - b) <= eps;

test('default scale is 1M/10k = 100x', () => {
  const r = projectSavingsTo1M({ wallBaseMs: 4, wallCandMs: 4, growthBase: 0.5, growthCand: 0.5 });
  assert.equal(r.scale, 100);
});

test('equal wall and equal exponent → zero projected saving', () => {
  const r = projectSavingsTo1M({ wallBaseMs: 4.082, wallCandMs: 4.082, growthBase: 0.361, growthCand: 0.361 });
  assert.ok(near(r.projSavings1MMs, 0), `expected ~0, got ${r.projSavings1MMs}`);
});

test('cache-cell-index-scatter: matches hand-computed ~3.30ms saving', () => {
  // base 3.126ms, cand 2.871ms at 10k; b 0.361 → 0.331. 100^0.361=5.272, 100^0.331=4.592.
  const r = projectSavingsTo1M({ wallBaseMs: 3.126, wallCandMs: 2.871, growthBase: 0.361, growthCand: 0.331 });
  assert.ok(near(r.projBase1MMs, 16.48, 0.05), `projBase ${r.projBase1MMs}`);
  assert.ok(near(r.projCand1MMs, 13.18, 0.05), `projCand ${r.projCand1MMs}`);
  assert.ok(near(r.projSavings1MMs, 3.30, 0.05), `saving ${r.projSavings1MMs}`);
});

test('a growth-class REGRESSION projects negative even when ~flat at 10k', () => {
  // histogram-buckets: nearly flat wall at 10k but b jumps 0.361 → 0.559.
  const r = projectSavingsTo1M({ wallBaseMs: 4.082, wallCandMs: 4.004, growthBase: 0.361, growthCand: 0.559 });
  assert.ok(r.projSavings1MMs < -10, `expected large negative, got ${r.projSavings1MMs}`);
});

test('lower candidate exponent widens the saving as pop grows', () => {
  const small = projectSavingsTo1M({ wallBaseMs: 4, wallCandMs: 4, growthBase: 0.5, growthCand: 0.4, toPop: 100000 });
  const big = projectSavingsTo1M({ wallBaseMs: 4, wallCandMs: 4, growthBase: 0.5, growthCand: 0.4, toPop: 1000000 });
  assert.ok(big.projSavings1MMs > small.projSavings1MMs, 'saving grows with target pop');
});

test('honors custom fromPop/toPop', () => {
  const r = projectSavingsTo1M({ wallBaseMs: 4, wallCandMs: 4, growthBase: 1, growthCand: 1, fromPop: 1000, toPop: 2000 });
  assert.equal(r.scale, 2);
  assert.ok(near(r.projBase1MMs, 8), `projBase ${r.projBase1MMs}`); // 4 * 2^1
});

test('rejects malformed input rather than emitting a bad projection', () => {
  assert.throws(() => projectSavingsTo1M({ wallBaseMs: 'x', wallCandMs: 4, growthBase: 0.3, growthCand: 0.3 }), /wallBaseMs/);
  assert.throws(() => projectSavingsTo1M({ wallBaseMs: 4, wallCandMs: 4, growthBase: NaN, growthCand: 0.3 }), /growthBase/);
  assert.throws(() => projectSavingsTo1M({ wallBaseMs: 4, wallCandMs: 4, growthBase: 0.3, growthCand: 0.3, toPop: 0 }), /positive/);
  assert.throws(() => projectSavingsTo1M(null), /required/);
});
