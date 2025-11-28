import type { CreatureData } from "@/types/GameState";

/**
 * Manages interleaved Float32Array buffer for GPU interpolation.
 *
 * Layout per creature (7 floats):
 * [startX, startY, endX, endY, startRot, endRot, size]
 *
 * START: Position/rotation from previous simulation tick
 * END: Position/rotation from current simulation tick
 * GPU vertex shader interpolates: mix(START, END, uInterpolation)
 *
 * Uses pre-allocated buffer with capacity to avoid GC pressure during spawning.
 */
export class InterpolationBufferManager {
  private static readonly FLOATS_PER_CREATURE = 7;
  private static readonly DEFAULT_CAPACITY = 200_000;

  private buffer: Float32Array;
  private capacity: number;
  private creatureCount: number = 0;
  private dirty: boolean = false;

  constructor(initialCapacity: number = InterpolationBufferManager.DEFAULT_CAPACITY) {
    this.capacity = initialCapacity;
    this.buffer = new Float32Array(this.capacity * InterpolationBufferManager.FLOATS_PER_CREATURE);
  }

  /**
   * Initialize buffer with creatures (first frame: START = END)
   */
  initialize(creatures: CreatureData[]): void {
    this.ensureCapacity(creatures.length);
    this.creatureCount = creatures.length;

    for (let i = 0; i < creatures.length; i++) {
      const c = creatures[i];
      const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

      this.buffer[offset + 0] = c.x; // startX
      this.buffer[offset + 1] = c.y; // startY
      this.buffer[offset + 2] = c.x; // endX (same)
      this.buffer[offset + 3] = c.y; // endY (same)
      this.buffer[offset + 4] = c.rotation; // startRot
      this.buffer[offset + 5] = c.rotation; // endRot (same)
      this.buffer[offset + 6] = c.size; // size
    }

    this.dirty = true;
  }

  /**
   * Update buffer on simulation tick (swap END → START, write new END)
   */
  update(newCreatures: CreatureData[]): void {
    const newCount = newCreatures.length;
    const oldCount = this.creatureCount;

    if (newCount !== oldCount) {
      this.resize(newCount, newCreatures);
      return;
    }

    for (let i = 0; i < newCount; i++) {
      const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

      this.buffer[offset + 0] = this.buffer[offset + 2]; // endX → startX
      this.buffer[offset + 1] = this.buffer[offset + 3]; // endY → startY
      this.buffer[offset + 4] = this.buffer[offset + 5]; // endRot → startRot
    }

    for (let i = 0; i < newCount; i++) {
      const c = newCreatures[i];
      const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

      this.buffer[offset + 2] = c.x; // endX
      this.buffer[offset + 3] = c.y; // endY
      this.buffer[offset + 5] = c.rotation; // endRot
      this.buffer[offset + 6] = c.size; // size
    }

    this.dirty = true;
  }

  /**
   * Ensure buffer has capacity for at least `requiredCount` creatures.
   * Only allocates if current capacity is insufficient.
   */
  private ensureCapacity(requiredCount: number): void {
    if (requiredCount <= this.capacity) {
      return;
    }

    const newCapacity = Math.max(requiredCount, this.capacity * 2);
    const newBuffer = new Float32Array(newCapacity * InterpolationBufferManager.FLOATS_PER_CREATURE);

    newBuffer.set(this.buffer);

    this.buffer = newBuffer;
    this.capacity = newCapacity;
  }

  /**
   * Handle creature count changes (spawn/despawn) - reuses existing buffer
   */
  private resize(newCount: number, newCreatures: CreatureData[]): void {
    const oldCount = this.creatureCount;

    this.ensureCapacity(newCount);
    this.creatureCount = newCount;

    if (newCount > oldCount) {
      for (let i = 0; i < oldCount; i++) {
        const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;
        const c = newCreatures[i];

        this.buffer[offset + 0] = this.buffer[offset + 2]; // endX → startX
        this.buffer[offset + 1] = this.buffer[offset + 3]; // endY → startY
        this.buffer[offset + 4] = this.buffer[offset + 5]; // endRot → startRot

        this.buffer[offset + 2] = c.x; // endX
        this.buffer[offset + 3] = c.y; // endY
        this.buffer[offset + 5] = c.rotation; // endRot
        this.buffer[offset + 6] = c.size; // size
      }

      for (let i = oldCount; i < newCount; i++) {
        const c = newCreatures[i];
        const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

        this.buffer[offset + 0] = c.x; // startX
        this.buffer[offset + 1] = c.y; // startY
        this.buffer[offset + 2] = c.x; // endX (same)
        this.buffer[offset + 3] = c.y; // endY (same)
        this.buffer[offset + 4] = c.rotation; // startRot
        this.buffer[offset + 5] = c.rotation; // endRot (same)
        this.buffer[offset + 6] = c.size; // size
      }
    } else {
      for (let i = 0; i < newCount; i++) {
        const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;
        const c = newCreatures[i];

        this.buffer[offset + 0] = this.buffer[offset + 2]; // endX → startX
        this.buffer[offset + 1] = this.buffer[offset + 3]; // endY → startY
        this.buffer[offset + 4] = this.buffer[offset + 5]; // endRot → startRot

        this.buffer[offset + 2] = c.x; // endX
        this.buffer[offset + 3] = c.y; // endY
        this.buffer[offset + 5] = c.rotation; // endRot
        this.buffer[offset + 6] = c.size; // size
      }
    }

    this.dirty = true;
  }

  /**
   * Get buffer view for GPU upload (only the used portion)
   */
  getBuffer(): Float32Array {
    const usedLength = this.creatureCount * InterpolationBufferManager.FLOATS_PER_CREATURE;
    return this.buffer.subarray(0, usedLength);
  }

  /**
   * Get current creature count
   */
  getCreatureCount(): number {
    return this.creatureCount;
  }

  /**
   * Get current buffer capacity (for testing)
   */
  getCapacity(): number {
    return this.capacity;
  }

  /**
   * Check if buffer has been updated since last markClean()
   */
  isDirty(): boolean {
    return this.dirty;
  }

  /**
   * Mark buffer as clean (after GPU upload)
   */
  markClean(): void {
    this.dirty = false;
  }
}
