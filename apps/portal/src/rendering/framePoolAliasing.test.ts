import { describe, it, expect } from 'vitest';
import { CreatureFramePool, type CreatureFrameSlot } from './CreatureFramePool';
import { SnapshotInterpolator } from './SnapshotInterpolator';

// Load-bearing invariant for the GC ring: a frame slot the renderer still references
// must NEVER be recycled by a later acquire(). The subtle live refs are the ones the
// renderer holds OUTSIDE the interpolator queue — `uploadedTo` (the `to` slot on the
// GPU) and its paired `from` — which lag the queue by up to a tick. If the pool is
// too small, a later acquire() recycles the slot uploadedTo still points at, and the
// GPU would interpolate against corrupted positions.
//
// This test models the renderer's exact produce→render→upload reference pattern under
// a delivery burst (pushes out-running renders) and asserts the slot uploadedTo/from
// reference keeps its fingerprint until it is replaced. The renderer sizes the pool to
// maxQueue + 2; these tests prove that is sufficient and that a too-small pool is not.

const TICK = 50;
const frame = (tick: number) => [{ id: tick, x: tick, y: 0, rotation: 0, size: 1 }];

/**
 * Drive the renderer's reference lifecycle and count aliasing violations: a frame the
 * GPU still references (uploadedTo / its from) whose fingerprint changed under it
 * because acquire() recycled the slot while it was still live.
 */
function countViolations(poolSize: number, maxQueue: number, ticks: number): number {
  const pool = new CreatureFramePool(poolSize, 4);
  const interp = new SnapshotInterpolator<CreatureFrameSlot>({ tickIntervalMs: TICK, maxQueue });
  let uploadedTo: CreatureFrameSlot | null = null;
  let uploadedFrom: CreatureFrameSlot | null = null;
  let toFp = 0;
  let fromFp = 0;
  let violations = 0;
  let t = 0;

  for (let i = 0; i < ticks; i++) {
    // Produce faster than we render (burst) to grow the queue and stress the lag.
    interp.push(pool.acquire(frame(t++)));
    interp.push(pool.acquire(frame(t++)));

    // Render: advance the clock, then "upload" when the displayed `to` changes.
    interp.advance(TICK);
    const seg = interp.current();
    if (!seg) continue;
    if (seg.to !== uploadedTo) {
      uploadedFrom = seg.from;
      uploadedTo = seg.to;
      fromFp = seg.from.xs[0]; // fingerprint at upload time
      toFp = seg.to.xs[0];
    }
    // The slots the GPU still references must not have been clobbered under us.
    if (uploadedTo!.xs[0] !== toFp) violations++;
    if (uploadedFrom!.xs[0] !== fromFp) violations++;
  }
  return violations;
}

describe('frame-pool aliasing invariant (renderer ref lifecycle)', () => {
  it('a pool sized maxQueue + 2 never clobbers a referenced frame', () => {
    expect(countViolations(8, 6, 200)).toBe(0); // the renderer's chosen sizing
  });

  it('a badly undersized pool DOES corrupt referenced frames — why headroom matters', () => {
    // Pool of 2 with a queue cap of 6: acquire() cycles every 2 pushes, so the slot
    // uploadedTo references is recycled almost immediately and its fingerprint changes.
    expect(countViolations(2, 6, 200)).toBeGreaterThan(0);
  });
});
