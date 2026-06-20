import { describe, it, expect } from 'vitest';
import { writeInterleavedSegment } from './interleavedBuffer';
import { CreatureFrameSlot } from './CreatureFramePool';

// Builds the shader's per-creature [startX,startY,endX,endY,startRot,endRot,size]
// buffer by interpolating between two SoA frame slots (from -> to), matched by
// creature id. The render set is `to`; a creature only in `to` (newly appeared) gets
// start=end (no ghosting); a creature only in `from` (departed) is dropped. Matching
// uses `from`'s PRE-BUILT idToIndex — the function allocates no Map of its own.

const FLOATS = 7;

/** Build a filled SoA slot from [id, x, y, rotation, size] tuples. */
const slot = (specs: Array<[number, number, number, number?, number?]>): CreatureFrameSlot =>
  new CreatureFrameSlot(Math.max(1, specs.length)).fill(
    specs.map(([id, x, y, rotation = 0, size = 1]) => ({ id, x, y, rotation, size }))
  );

describe('writeInterleavedSegment (SoA slots)', () => {
  it('uses from-position as start and to-position as end for a matched creature', () => {
    const out = new Float32Array(FLOATS);
    const count = writeInterleavedSegment(slot([[7, 0, 0, 0]]), slot([[7, 10, 20, 1.5]]), out);
    expect(count).toBe(1);
    expect(Array.from(out)).toEqual([
      0, 0, // startX, startY (from)
      10, 20, // endX, endY (to)
      0, 1.5, // startRot (from), endRot (to)
      1, // size (to)
    ]);
  });

  it('gives a newly-appeared creature start = end (no ghosting)', () => {
    const out = new Float32Array(FLOATS);
    // id 9 is in `to` but not `from`
    writeInterleavedSegment(slot([[7, 0, 0]]), slot([[9, 5, 6, 2, 3]]), out);
    expect(Array.from(out)).toEqual([5, 6, 5, 6, 2, 2, 3]); // start == end
  });

  it('renders the `to` set and drops creatures that departed', () => {
    const out = new Float32Array(2 * FLOATS);
    // from has ids 1,2; to has ids 2,3 — id 1 departed, id 3 appeared
    const count = writeInterleavedSegment(
      slot([[1, 0, 0], [2, 100, 100]]),
      slot([[2, 110, 110], [3, 0, 0]]),
      out
    );
    expect(count).toBe(2); // only the `to` set
    // creature 2 (matched): start = from(100,100), end = to(110,110)
    expect(out[0]).toBe(100);
    expect(out[1]).toBe(100);
    expect(out[2]).toBe(110);
    expect(out[3]).toBe(110);
    // creature 3 (new): start == end
    expect(out[FLOATS + 0]).toBe(0);
    expect(out[FLOATS + 2]).toBe(0);
  });

  it('matches by id regardless of array order', () => {
    const out = new Float32Array(2 * FLOATS);
    writeInterleavedSegment(
      slot([[1, 10, 0], [2, 20, 0]]),
      slot([[2, 21, 0], [1, 11, 0]]), // reversed order
      out
    );
    // to[0] is id 2 -> start from id 2 (20), end 21
    expect(out[0]).toBe(20);
    expect(out[2]).toBe(21);
    // to[1] is id 1 -> start from id 1 (10), end 11
    expect(out[FLOATS + 0]).toBe(10);
    expect(out[FLOATS + 2]).toBe(11);
  });

  it('matches via `from`\'s pre-built idToIndex (builds no Map of its own)', () => {
    const out = new Float32Array(FLOATS);
    const from = slot([[5, 100, 100]]);
    // Drop id 5 from the pre-built map: if the function consults THIS map (not the
    // raw ids array), id 5 now reads as "new" → start = end. If it rebuilt its own
    // map from `from.ids`, it would still find id 5 and use start = 100.
    from.idToIndex.delete(5);
    writeInterleavedSegment(from, slot([[5, 110, 110]]), out);
    expect(out[0]).toBe(110); // startX == endX → treated as new (used the prebuilt map)
    expect(out[2]).toBe(110);
  });
});
