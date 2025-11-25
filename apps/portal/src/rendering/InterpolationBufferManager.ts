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
 */
export class InterpolationBufferManager {
  private static readonly FLOATS_PER_CREATURE = 7;

  private buffer: Float32Array;
  private creatureCount: number = 0;
  private dirty: boolean = false;

  constructor() {
    this.buffer = new Float32Array(0);
  }

  /**
   * Initialize buffer with creatures (first frame: START = END)
   */
  initialize(creatures: CreatureData[]): void {
    this.creatureCount = creatures.length;
    const size =
      this.creatureCount * InterpolationBufferManager.FLOATS_PER_CREATURE;

    // Allocate or resize buffer
    if (this.buffer.length !== size) {
      this.buffer = new Float32Array(size);
    }

    // Initialize: START = END (no interpolation on first frame)
    for (let i = 0; i < creatures.length; i++) {
      const c = creatures[i];
      const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

      // START and END are identical initially
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

    // Handle creature count changes
    if (newCount !== this.creatureCount) {
      this.resize(newCount, newCreatures);
      return;
    }

    // Same count: swap END → START, write new END
    for (let i = 0; i < newCount; i++) {
      const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

      // Copy END → START
      this.buffer[offset + 0] = this.buffer[offset + 2]; // endX → startX
      this.buffer[offset + 1] = this.buffer[offset + 3]; // endY → startY
      this.buffer[offset + 4] = this.buffer[offset + 5]; // endRot → startRot
    }

    // Write new END positions
    for (let i = 0; i < newCount; i++) {
      const c = newCreatures[i];
      const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

      this.buffer[offset + 2] = c.x; // endX
      this.buffer[offset + 3] = c.y; // endY
      this.buffer[offset + 5] = c.rotation; // endRot
      this.buffer[offset + 6] = c.size; // size (can change with growth)
    }

    this.dirty = true;
  }

  /**
   * Resize buffer when creature count changes (spawn/despawn)
   */
  private resize(newCount: number, newCreatures: CreatureData[]): void {
    const oldCount = this.creatureCount;
    this.creatureCount = newCount;

    const newSize =
      newCount * InterpolationBufferManager.FLOATS_PER_CREATURE;

    // Allocate new buffer
    const newBuffer = new Float32Array(newSize);

    if (newCount > oldCount) {
      // Growing: copy existing, initialize new creatures with START = END
      // Copy existing creatures (preserve START/END)
      const copyCount = Math.min(oldCount, newCount);
      for (let i = 0; i < copyCount; i++) {
        const oldOffset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;
        const newOffset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

        // Copy all 7 floats
        for (let j = 0; j < InterpolationBufferManager.FLOATS_PER_CREATURE; j++) {
          newBuffer[newOffset + j] = this.buffer[oldOffset + j];
        }

        // Update END with new data
        const c = newCreatures[i];
        newBuffer[newOffset + 2] = c.x; // endX
        newBuffer[newOffset + 3] = c.y; // endY
        newBuffer[newOffset + 5] = c.rotation; // endRot
        newBuffer[newOffset + 6] = c.size; // size
      }

      // Initialize newly spawned creatures (START = END)
      for (let i = oldCount; i < newCount; i++) {
        const c = newCreatures[i];
        const offset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;

        newBuffer[offset + 0] = c.x; // startX
        newBuffer[offset + 1] = c.y; // startY
        newBuffer[offset + 2] = c.x; // endX (same)
        newBuffer[offset + 3] = c.y; // endY (same)
        newBuffer[offset + 4] = c.rotation; // startRot
        newBuffer[offset + 5] = c.rotation; // endRot (same)
        newBuffer[offset + 6] = c.size; // size
      }
    } else {
      // Shrinking: copy subset
      for (let i = 0; i < newCount; i++) {
        const oldOffset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;
        const newOffset = i * InterpolationBufferManager.FLOATS_PER_CREATURE;
        const c = newCreatures[i];

        // Swap END → START
        newBuffer[newOffset + 0] = this.buffer[oldOffset + 2]; // endX → startX
        newBuffer[newOffset + 1] = this.buffer[oldOffset + 3]; // endY → startY
        newBuffer[newOffset + 4] = this.buffer[oldOffset + 5]; // endRot → startRot

        // Write new END
        newBuffer[newOffset + 2] = c.x; // endX
        newBuffer[newOffset + 3] = c.y; // endY
        newBuffer[newOffset + 5] = c.rotation; // endRot
        newBuffer[newOffset + 6] = c.size; // size
      }
    }

    this.buffer = newBuffer;
    this.dirty = true;
  }

  /**
   * Get read-only access to buffer (for GPU upload)
   */
  getBuffer(): Float32Array {
    return this.buffer;
  }

  /**
   * Get current creature count
   */
  getCreatureCount(): number {
    return this.creatureCount;
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
