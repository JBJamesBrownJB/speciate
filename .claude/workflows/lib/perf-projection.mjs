// Growth-exponent-aware projection of a cloud-triage wall-time saving up to a
// target population (default 1M). This is the SOURCE OF TRUTH for the formula the
// perf-hunt-cloud workflow inlines in its Log phase (the workflow sandbox has no
// module import, so it carries an identical `Math.pow` expression — keep them in
// sync; node --test guards the contract here).
//
// WHY growth-aware, not linear: the cloud hunt measures at ~10k. A candidate can
// look flat at 10k yet change the SCALING CLASS (its fitted growth exponent b),
// which dominates at 1M. Projecting each side by its own exponent surfaces that:
//   proj(pop) = wall_10k * (pop / from_pop)^b
//   saving    = proj_base(1M) - proj_cand(1M)   // positive = candidate faster
//
// CAVEAT (callers MUST surface this): 100x extrapolation of a noisy fitted b is
// wildly sensitive — 100^0.36 ~ 5.3x vs 100^0.56 ~ 13x from a 0.2 swing in b. The
// SIGN and ORDER OF MAGNITUDE are the signal; the digits are not a prediction.

const DEFAULT_FROM_POP = 10000;
const DEFAULT_TO_POP = 1000000;

function num(v, who, name) {
  if (typeof v !== 'number' || !Number.isFinite(v)) throw new Error(`${who}: ${name} must be a finite number`);
  return v;
}

/**
 * Project a per-candidate wall-time saving from the cloud test pop up to a target
 * population using each side's fitted growth exponent.
 *
 * @param {object} f
 * @param {number} f.wallBaseMs   baseline wall (p99) at the cloud test pop, ms
 * @param {number} f.wallCandMs   candidate wall (p99) at the cloud test pop, ms
 * @param {number} f.growthBase   fitted growth exponent b of the baseline sweep
 * @param {number} f.growthCand   fitted growth exponent b of the candidate sweep
 * @param {number} [f.fromPop=10000]  population the wall figures were measured at
 * @param {number} [f.toPop=1000000]  population to project to
 * @returns {{scale:number, projBase1MMs:number, projCand1MMs:number, projSavings1MMs:number}}
 *   projSavings1MMs is positive when the candidate is projected FASTER at toPop.
 */
export function projectSavingsTo1M(f) {
  if (!f || typeof f !== 'object') throw new Error('projectSavingsTo1M: fields object required');
  const wallBaseMs = num(f.wallBaseMs, 'projectSavingsTo1M', 'wallBaseMs');
  const wallCandMs = num(f.wallCandMs, 'projectSavingsTo1M', 'wallCandMs');
  const growthBase = num(f.growthBase, 'projectSavingsTo1M', 'growthBase');
  const growthCand = num(f.growthCand, 'projectSavingsTo1M', 'growthCand');
  const fromPop = f.fromPop === undefined ? DEFAULT_FROM_POP : num(f.fromPop, 'projectSavingsTo1M', 'fromPop');
  const toPop = f.toPop === undefined ? DEFAULT_TO_POP : num(f.toPop, 'projectSavingsTo1M', 'toPop');
  if (fromPop <= 0 || toPop <= 0) throw new Error('projectSavingsTo1M: populations must be positive');

  const scale = toPop / fromPop;
  const projBase1MMs = wallBaseMs * Math.pow(scale, growthBase);
  const projCand1MMs = wallCandMs * Math.pow(scale, growthCand);
  return {
    scale,
    projBase1MMs,
    projCand1MMs,
    projSavings1MMs: projBase1MMs - projCand1MMs,
  };
}
