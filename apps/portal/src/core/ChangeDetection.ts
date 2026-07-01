import type { CreatureData } from "@/types/GameState";

const FLOATS_PER_CREATURE = 5; // id, x, y, rotation, size

/**
 * Decides whether a delivered frame carries new data (worth re-tweening the
 * renderer) or is a duplicate.
 *
 * Push-on-swap path (tick > 0): each delivery corresponds to one Rust buffer
 * swap, so tick identity IS the change signal — O(1), no scanning.
 *
 * Poll-fallback path (tick 0, addon without the doorbell): exact value
 * comparison against a pooled snapshot of the previous frame. Exact on purpose:
 * the old first-3/last-3 sampled hash could freeze a mid-array mover when the
 * sampled creatures were catatonic. The compare buffer is reused (grown
 * geometrically) — no per-tick allocation in steady state.
 */
export class ChangeDetector {
  private lastTick = -1;

  private prev: Float32Array = new Float32Array(0);
  private prevCount = -1; // -1 = nothing seen yet, so the first frame always updates

  shouldUpdate(creatures: CreatureData[], tick: number): boolean {
    if (tick > 0) {
      const changed = tick !== this.lastTick;
      this.lastTick = tick;
      return changed;
    }
    return this.compareExact(creatures);
  }

  private compareExact(creatures: CreatureData[]): boolean {
    const count = creatures.length;
    const changed = count !== this.prevCount || !this.valuesEqual(creatures);

    if (changed) {
      this.snapshot(creatures);
    }
    return changed;
  }

  private valuesEqual(creatures: CreatureData[]): boolean {
    const prev = this.prev;
    for (let i = 0; i < creatures.length; i++) {
      const c = creatures[i];
      const o = i * FLOATS_PER_CREATURE;
      if (
        prev[o] !== c.id ||
        prev[o + 1] !== c.x ||
        prev[o + 2] !== c.y ||
        prev[o + 3] !== c.rotation ||
        prev[o + 4] !== c.size
      ) {
        return false;
      }
    }
    return true;
  }

  private snapshot(creatures: CreatureData[]): void {
    const needed = creatures.length * FLOATS_PER_CREATURE;
    if (this.prev.length < needed) {
      this.prev = new Float32Array(Math.max(needed, this.prev.length * 2));
    }
    const prev = this.prev;
    for (let i = 0; i < creatures.length; i++) {
      const c = creatures[i];
      const o = i * FLOATS_PER_CREATURE;
      prev[o] = c.id;
      prev[o + 1] = c.x;
      prev[o + 2] = c.y;
      prev[o + 3] = c.rotation;
      prev[o + 4] = c.size;
    }
    this.prevCount = creatures.length;
  }

  reset(): void {
    this.lastTick = -1;
    this.prevCount = -1;
  }
}
