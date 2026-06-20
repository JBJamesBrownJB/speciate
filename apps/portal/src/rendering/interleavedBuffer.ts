import type { CreatureFrameSlot } from './CreatureFramePool';

/** Floats per creature in the interleaved start/end buffer the shader consumes. */
export const FLOATS_PER_CREATURE = 7;

/**
 * Build the shader's per-creature interleaved buffer by interpolating between two
 * SoA frame slots (from -> to), matched by creature id. Render set is `to`. A
 * creature present only in `to` (newly appeared) gets start = end (no ghosting); a
 * creature present only in `from` (departed) is dropped. Layout per creature:
 * [startX, startY, endX, endY, startRot, endRot, size]. Returns the count written.
 *
 * Matching uses `from`'s PRE-BUILT `idToIndex` (populated by CreatureFrameSlot.fill),
 * so this hot path allocates nothing — no per-call Map, no per-creature objects.
 */
export function writeInterleavedSegment(
  from: CreatureFrameSlot,
  to: CreatureFrameSlot,
  out: Float32Array
): number {
  const fromIndex = from.idToIndex;

  for (let i = 0; i < to.count; i++) {
    const o = i * FLOATS_PER_CREATURE;
    const j = fromIndex.get(to.ids[i]);

    if (j !== undefined) {
      out[o + 0] = from.xs[j]; // startX = from
      out[o + 1] = from.ys[j]; // startY = from
      out[o + 4] = from.rots[j]; // startRot = from
    } else {
      out[o + 0] = to.xs[i]; // new creature: start = end (no interp / no ghost)
      out[o + 1] = to.ys[i];
      out[o + 4] = to.rots[i];
    }

    out[o + 2] = to.xs[i]; // endX = to
    out[o + 3] = to.ys[i]; // endY = to
    out[o + 5] = to.rots[i]; // endRot = to
    out[o + 6] = to.sizes[i]; // size = to
  }

  return to.count;
}
