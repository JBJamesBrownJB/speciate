interface CreatureLike {
  id: number;
  x: number;
  y: number;
  rotation: number;
  size: number;
}

/** Floats per creature in the interleaved start/end buffer the shader consumes. */
export const FLOATS_PER_CREATURE = 7;

/**
 * Build the shader's per-creature interleaved buffer by interpolating between two
 * snapshots (from -> to), matched by creature id. Render set is `to`. A creature
 * present only in `to` (newly appeared) gets start = end (no ghosting); a creature
 * present only in `from` (departed) is dropped. Layout per creature:
 * [startX, startY, endX, endY, startRot, endRot, size]. Returns the count written.
 */
export function writeInterleavedSegment(
  from: CreatureLike[],
  to: CreatureLike[],
  out: Float32Array
): number {
  const fromIndex = new Map<number, number>();
  for (let i = 0; i < from.length; i++) fromIndex.set(from[i].id, i);

  for (let i = 0; i < to.length; i++) {
    const c = to[i];
    const o = i * FLOATS_PER_CREATURE;
    const j = fromIndex.get(c.id);

    if (j !== undefined) {
      out[o + 0] = from[j].x; // startX = from
      out[o + 1] = from[j].y; // startY = from
      out[o + 4] = from[j].rotation; // startRot = from
    } else {
      out[o + 0] = c.x; // new creature: start = end (no interp / no ghost)
      out[o + 1] = c.y;
      out[o + 4] = c.rotation;
    }

    out[o + 2] = c.x; // endX = to
    out[o + 3] = c.y; // endY = to
    out[o + 5] = c.rotation; // endRot = to
    out[o + 6] = c.size; // size = to
  }

  return to.length;
}
