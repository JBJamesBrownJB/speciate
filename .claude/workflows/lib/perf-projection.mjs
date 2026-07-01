// Growth-aware projection of a cloud-triage wall saving to a target population.
// WHY: the hunt measures at ~10k, where an idea that changes the SCALING CLASS
// (its fitted exponent b) can look flat yet win/regress at 1M. Scaling each side
// by its own b surfaces that. Tested source of truth for the formula perf-hunt-
// cloud.mjs inlines (the workflow sandbox can't import) — keep the two in sync.
// CAVEAT for callers: 100^b is very sensitive to a noisy b; sign and order of
// magnitude are the signal, not the digits.

const num = (v, name) => {
  if (typeof v !== 'number' || !Number.isFinite(v)) throw new Error(`projectSavingsTo1M: ${name} must be a finite number`);
  return v;
};

/**
 * @param {{wallBaseMs:number, wallCandMs:number, growthBase:number, growthCand:number, fromPop?:number, toPop?:number}} f
 *   wall figures are p99 at the cloud test pop (fromPop, default 10k); growth* are
 *   the fitted sweep exponents. Projects to toPop (default 1M).
 * @returns {{scale:number, projBase1MMs:number, projCand1MMs:number, projSavings1MMs:number}}
 *   projSavings1MMs > 0 ⇒ candidate projected FASTER at toPop.
 */
export function projectSavingsTo1M(f) {
  if (!f || typeof f !== 'object') throw new Error('projectSavingsTo1M: fields object required');
  const fromPop = f.fromPop === undefined ? 10000 : num(f.fromPop, 'fromPop');
  const toPop = f.toPop === undefined ? 1000000 : num(f.toPop, 'toPop');
  if (fromPop <= 0 || toPop <= 0) throw new Error('projectSavingsTo1M: populations must be positive');

  const scale = toPop / fromPop;
  const projBase1MMs = num(f.wallBaseMs, 'wallBaseMs') * Math.pow(scale, num(f.growthBase, 'growthBase'));
  const projCand1MMs = num(f.wallCandMs, 'wallCandMs') * Math.pow(scale, num(f.growthCand, 'growthCand'));
  return { scale, projBase1MMs, projCand1MMs, projSavings1MMs: projBase1MMs - projCand1MMs };
}
