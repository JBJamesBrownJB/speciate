import { describe, it, expect } from 'vitest';
import { CreatureFramePool } from './CreatureFramePool';
import type { CreatureData } from '@/types/GameState';
import { FLOATS_PER_CREATURE, getBufferOffsets } from '@/types/BufferLayout';

// The GC ring: instead of allocating a fresh object[] snapshot every tick (10M+
// short-lived objects/sec at 500k creatures), the renderer fills a round-robin pool
// of pre-allocated Structure-of-Arrays slots. The contract these tests pin down:
//   - SoA copy is faithful (ids + per-field typed arrays + count)
//   - idToIndex is rebuilt on every fill (no stale ids leak across refills)
//   - the pool hands out slots round-robin (so older frames stay live in the queue)
//   - refilling within capacity allocates NOTHING (the GC win — same backing arrays)

const make = (specs: Array<[number, number, number, number, number]>): CreatureData[] =>
  specs.map(([id, x, y, rotation, size]) => ({ id, x, y, rotation, size }));

describe('CreatureFramePool — SoA frame slots (GC ring)', () => {
  it('copies a frame into SoA typed arrays faithfully', () => {
    const pool = new CreatureFramePool(4, 16);
    const slot = pool.acquire(make([
      [7, 1, 2, 0.5, 10],
      [9, 3, 4, 1.5, 12],
    ]));

    expect(slot.count).toBe(2);
    expect(Array.from(slot.ids.subarray(0, 2))).toEqual([7, 9]);
    expect(Array.from(slot.xs.subarray(0, 2))).toEqual([1, 3]);
    expect(Array.from(slot.ys.subarray(0, 2))).toEqual([2, 4]);
    expect(Array.from(slot.rots.subarray(0, 2))).toEqual([0.5, 1.5]);
    expect(Array.from(slot.sizes.subarray(0, 2))).toEqual([10, 12]);
    expect(slot.idToIndex.get(7)).toBe(0);
    expect(slot.idToIndex.get(9)).toBe(1);
  });

  it('rebuilds idToIndex on refill so stale ids never leak', () => {
    const pool = new CreatureFramePool(1, 16); // single slot, reused every acquire
    pool.acquire(make([[1, 0, 0, 0, 1], [2, 0, 0, 0, 1], [3, 0, 0, 0, 1]]));
    const slot = pool.acquire(make([[4, 0, 0, 0, 1], [5, 0, 0, 0, 1]]));

    expect(slot.count).toBe(2);
    expect(slot.idToIndex.size).toBe(2);
    expect(slot.idToIndex.has(1)).toBe(false); // stale id from prior fill gone
    expect(slot.idToIndex.get(4)).toBe(0);
    expect(slot.idToIndex.get(5)).toBe(1);
  });

  it('hands out slots round-robin and wraps after poolSize', () => {
    const pool = new CreatureFramePool(3, 8);
    const a = pool.acquire(make([[1, 0, 0, 0, 1]]));
    const b = pool.acquire(make([[2, 0, 0, 0, 1]]));
    const c = pool.acquire(make([[3, 0, 0, 0, 1]]));
    expect(a).not.toBe(b);
    expect(b).not.toBe(c);
    expect(a).not.toBe(c);

    const d = pool.acquire(make([[4, 0, 0, 0, 1]])); // wraps back to the first slot
    expect(d).toBe(a);
  });

  it('refilling within capacity reuses the SAME backing arrays (no alloc — the GC win)', () => {
    const pool = new CreatureFramePool(1, 16);
    const first = pool.acquire(make([[1, 0, 0, 0, 1]]));
    const xs = first.xs;
    const ids = first.ids;

    const second = pool.acquire(make([[2, 9, 9, 9, 9], [3, 8, 8, 8, 8]]));
    expect(second).toBe(first);
    expect(second.xs).toBe(xs); // identical reference — nothing was reallocated
    expect(second.ids).toBe(ids);
  });

  it('grows a slot when a frame exceeds capacity (no silent truncation)', () => {
    const pool = new CreatureFramePool(2, 2); // capacity 2
    const slot = pool.acquire(make([
      [1, 0, 0, 0, 1], [2, 0, 0, 0, 1], [3, 0, 0, 0, 1], [4, 0, 0, 0, 1],
    ]));
    expect(slot.count).toBe(4);
    expect(Array.from(slot.ids.subarray(0, 4))).toEqual([1, 2, 3, 4]);
    expect(slot.xs.length).toBeGreaterThanOrEqual(4);
  });

  describe('SoA fast path (fill straight from the IPC buffer — no object hop)', () => {
    /** Build the wire-format SoA buffer ([IDs..., Xs..., Ys..., Rots..., Sizes...]). */
    function soaBuffer(creatures: CreatureData[]): Float32Array {
      const n = creatures.length;
      const buf = new Float32Array(n * FLOATS_PER_CREATURE);
      const o = getBufferOffsets(n);
      creatures.forEach((c, i) => {
        buf[o.id + i] = c.id;
        buf[o.x + i] = c.x;
        buf[o.y + i] = c.y;
        buf[o.rot + i] = c.rotation;
        buf[o.size + i] = c.size;
      });
      return buf;
    }

    it('acquireFromSoA yields a slot identical to the object path (values, not just shape)', () => {
      const creatures = make([
        [7, 1.5, 2.5, 0.5, 10],
        [9, 3.5, 4.5, 1.5, 12],
        [11, -8, 6, -0.25, 3],
      ]);
      const objPool = new CreatureFramePool(1, 16);
      const soaPool = new CreatureFramePool(1, 16);

      const fromObjects = objPool.acquire(creatures);
      const fromSoA = soaPool.acquireFromSoA(soaBuffer(creatures), creatures.length);

      expect(fromSoA.count).toBe(fromObjects.count);
      for (const field of ['ids', 'xs', 'ys', 'rots', 'sizes'] as const) {
        expect(Array.from(fromSoA[field].subarray(0, 3))).toEqual(
          Array.from(fromObjects[field].subarray(0, 3))
        );
      }
      expect(fromSoA.idToIndex.get(11)).toBe(2);
    });

    it('handles a zero-count buffer (empty world)', () => {
      const pool = new CreatureFramePool(1, 4);
      pool.acquireFromSoA(soaBuffer(make([[1, 0, 0, 0, 1]])), 1);
      const slot = pool.acquireFromSoA(new Float32Array(0), 0);
      expect(slot.count).toBe(0);
      expect(slot.idToIndex.size).toBe(0);
    });

    it('grows when the frame exceeds capacity, and reuses arrays within it', () => {
      const pool = new CreatureFramePool(1, 2);
      const big = pool.acquireFromSoA(
        soaBuffer(make([[1, 0, 0, 0, 1], [2, 0, 0, 0, 1], [3, 5, 6, 0, 1]])), 3
      );
      expect(big.count).toBe(3);
      expect(big.xs[2]).toBe(5);

      const xs = big.xs;
      const again = pool.acquireFromSoA(soaBuffer(make([[4, 9, 9, 0, 1]])), 1);
      expect(again.xs).toBe(xs); // same backing array — no realloc within capacity
      expect(again.idToIndex.has(1)).toBe(false);
    });
  });
});
