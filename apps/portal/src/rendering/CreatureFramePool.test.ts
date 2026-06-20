import { describe, it, expect } from 'vitest';
import { CreatureFramePool } from './CreatureFramePool';
import type { CreatureData } from '@/types/GameState';

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
});
