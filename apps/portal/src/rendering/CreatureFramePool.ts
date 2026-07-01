import { getBufferOffsets } from "@/types/BufferLayout";

interface CreatureLike {
  id: number;
  x: number;
  y: number;
  rotation: number;
  size: number;
}

/**
 * One frame's creatures in Structure-of-Arrays form. Position/rotation/size live in
 * parallel typed arrays sized to capacity; `count` is the live length. `idToIndex`
 * is rebuilt on every fill so the interleaved-buffer builder can match `from`→`to`
 * by id without allocating a Map per draw.
 *
 * The point is reuse: `fill()` writes through the existing typed arrays and never
 * allocates per-creature objects (the per-tick GC churn we're eliminating). It only
 * reallocates when a frame is larger than the current capacity.
 */
export class CreatureFrameSlot {
  ids: Int32Array;
  xs: Float32Array;
  ys: Float32Array;
  rots: Float32Array;
  sizes: Float32Array;
  count = 0;
  readonly idToIndex = new Map<number, number>();

  constructor(capacity: number) {
    this.ids = new Int32Array(capacity);
    this.xs = new Float32Array(capacity);
    this.ys = new Float32Array(capacity);
    this.rots = new Float32Array(capacity);
    this.sizes = new Float32Array(capacity);
  }

  /** Grow the backing arrays to hold at least `capacity` entries. No-op if they
   *  already fit, so a steady-state fill reuses the same arrays (the GC win). */
  ensureCapacity(capacity: number): void {
    if (capacity <= this.ids.length) return;
    this.ids = new Int32Array(capacity);
    this.xs = new Float32Array(capacity);
    this.ys = new Float32Array(capacity);
    this.rots = new Float32Array(capacity);
    this.sizes = new Float32Array(capacity);
  }

  /**
   * Fill straight from the wire-format SoA buffer ([IDs..., Xs..., Ys...,
   * Rots..., Sizes...] — src/types/BufferLayout.ts). The buffer is already the
   * layout we want, so each column is one typed-array copy; only ids take a
   * loop (Int32 conversion + idToIndex rebuild). This is the production path —
   * no per-creature object ever materializes between IPC and GPU.
   */
  fillFromSoA(buffer: Float32Array, count: number): this {
    this.ensureCapacity(count);
    const o = getBufferOffsets(count);
    this.xs.set(buffer.subarray(o.x, o.x + count));
    this.ys.set(buffer.subarray(o.y, o.y + count));
    this.rots.set(buffer.subarray(o.rot, o.rot + count));
    this.sizes.set(buffer.subarray(o.size, o.size + count));
    this.idToIndex.clear();
    for (let i = 0; i < count; i++) {
      const id = buffer[o.id + i];
      this.ids[i] = id;
      this.idToIndex.set(id, i);
    }
    this.count = count;
    return this;
  }

  /** Copy a frame in via typed-array writes only (no per-creature allocation) and
   *  rebuild idToIndex. Returns itself for convenient chaining from the pool. */
  fill(creatures: CreatureLike[]): this {
    const n = creatures.length;
    this.ensureCapacity(n);
    this.idToIndex.clear();
    for (let i = 0; i < n; i++) {
      const c = creatures[i];
      this.ids[i] = c.id;
      this.xs[i] = c.x;
      this.ys[i] = c.y;
      this.rots[i] = c.rotation;
      this.sizes[i] = c.size;
      this.idToIndex.set(c.id, i);
    }
    this.count = n;
    return this;
  }
}

/**
 * Round-robin pool of pre-allocated SoA frame slots. The renderer acquires a slot
 * per snapshot instead of allocating a fresh object[] each tick. Round-robin (not
 * a free list) is deliberate: older frames stay live in the interpolator's queue,
 * so the pool must hand out a *different* slot each tick and only recycle one once
 * it's no longer referenced. Sizing invariant the renderer enforces:
 * poolSize >= maxQueue + 2 (the +2 covers the lagging latestSnapshot/uploadedTo).
 */
export class CreatureFramePool {
  private readonly slots: CreatureFrameSlot[];
  private next = 0;

  constructor(poolSize: number, capacity: number) {
    this.slots = Array.from({ length: poolSize }, () => new CreatureFrameSlot(capacity));
  }

  /** Fill the next slot in the ring with `creatures` and return it. */
  acquire(creatures: CreatureLike[]): CreatureFrameSlot {
    const slot = this.slots[this.next];
    this.next = (this.next + 1) % this.slots.length;
    return slot.fill(creatures);
  }

  /** Fill the next slot straight from the wire-format SoA buffer. */
  acquireFromSoA(buffer: Float32Array, count: number): CreatureFrameSlot {
    const slot = this.slots[this.next];
    this.next = (this.next + 1) % this.slots.length;
    return slot.fillFromSoA(buffer, count);
  }

  /** Pre-grow every slot so an upcoming large frame never reallocates mid-ring. */
  ensureCapacity(capacity: number): void {
    for (const slot of this.slots) slot.ensureCapacity(capacity);
  }
}
