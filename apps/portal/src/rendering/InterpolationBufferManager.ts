import type { CreatureData } from "@/types/GameState";

interface InterpolationState {
  endX: number;
  endY: number;
  endRot: number;
  size: number;
}

export class InterpolationBufferManager {
  private static readonly FLOATS_PER_CREATURE = 7;
  private static readonly DEFAULT_CAPACITY = 200_000;

  private buffer: Float32Array;
  private capacity: number;
  private creatureCount: number = 0;
  private dirty: boolean = false;

  private stateById: Map<number, InterpolationState> = new Map();
  private visibleLastTick: Set<number> = new Set();

  constructor(initialCapacity: number = InterpolationBufferManager.DEFAULT_CAPACITY) {
    this.capacity = initialCapacity;
    this.buffer = new Float32Array(this.capacity * InterpolationBufferManager.FLOATS_PER_CREATURE);
  }

  initialize(creatures: CreatureData[]): void {
    this.ensureCapacity(creatures.length);
    this.creatureCount = creatures.length;
    this.stateById.clear();
    this.visibleLastTick.clear();

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

      this.stateById.set(c.id, {
        endX: c.x,
        endY: c.y,
        endRot: c.rotation,
        size: c.size,
      });
      this.visibleLastTick.add(c.id);
    }

    this.dirty = true;
  }

  update(newCreatures: CreatureData[]): void {
    const newCount = newCreatures.length;
    this.ensureCapacity(newCount);
    this.creatureCount = newCount;

    const visibleThisTick = new Set<number>();

    for (let i = 0; i < newCount; i++) {
      const c = newCreatures[i];
      const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

      // Only use previous state if creature was visible LAST tick (not stale cache)
      const wasVisibleLastTick = this.visibleLastTick.has(c.id);
      const prevState = wasVisibleLastTick ? this.stateById.get(c.id) : undefined;

      if (prevState) {
        this.buffer[offset + 0] = prevState.endX; // startX = previous endX
        this.buffer[offset + 1] = prevState.endY; // startY = previous endY
        this.buffer[offset + 4] = prevState.endRot; // startRot = previous endRot
      } else {
        this.buffer[offset + 0] = c.x; // startX = new position (no interpolation)
        this.buffer[offset + 1] = c.y; // startY
        this.buffer[offset + 4] = c.rotation; // startRot
      }

      this.buffer[offset + 2] = c.x; // endX
      this.buffer[offset + 3] = c.y; // endY
      this.buffer[offset + 5] = c.rotation; // endRot
      this.buffer[offset + 6] = c.size; // size

      this.stateById.set(c.id, {
        endX: c.x,
        endY: c.y,
        endRot: c.rotation,
        size: c.size,
      });
      visibleThisTick.add(c.id);
    }

    // Clean up stateById to only keep current tick's creatures
    // (prevents stale data if creature IDs are reused after death)
    for (const id of this.stateById.keys()) {
      if (!visibleThisTick.has(id)) {
        this.stateById.delete(id);
      }
    }

    this.visibleLastTick = visibleThisTick;
    this.dirty = true;
  }

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

  getBuffer(): Float32Array {
    const usedLength = this.creatureCount * InterpolationBufferManager.FLOATS_PER_CREATURE;
    return this.buffer.subarray(0, usedLength);
  }

  getCreatureCount(): number {
    return this.creatureCount;
  }

  getCapacity(): number {
    return this.capacity;
  }

  isDirty(): boolean {
    return this.dirty;
  }

  markClean(): void {
    this.dirty = false;
  }
}
