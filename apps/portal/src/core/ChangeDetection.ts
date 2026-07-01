import { FLOATS_PER_CREATURE } from "@/types/BufferLayout";

/**
 * Decides whether a delivered frame carries new data (worth re-tweening the
 * renderer) or is a duplicate.
 *
 * Push-on-swap path (tick > 0): each delivery corresponds to one Rust buffer
 * swap, so tick identity IS the change signal — O(1), no scanning.
 *
 * Poll-fallback path (tick 0, addon without the doorbell): exact comparison of
 * the wire SoA buffer against a pooled snapshot of the previous frame. Exact
 * on purpose: the old first-3/last-3 sampled hash could freeze a mid-array
 * mover when the sampled creatures were catatonic. The compare buffer is
 * reused (grown geometrically) — no per-tick allocation in steady state.
 */
export class ChangeDetector {
  private lastTick = -1;

  private prev: Float32Array = new Float32Array(0);
  private prevCount = -1; // -1 = nothing seen yet, so the first frame always updates

  shouldUpdate(tick: number, buffer: Float32Array, count: number): boolean {
    if (tick > 0) {
      const changed = tick !== this.lastTick;
      this.lastTick = tick;
      return changed;
    }
    return this.compareExact(buffer, count);
  }

  private compareExact(buffer: Float32Array, count: number): boolean {
    const floats = count * FLOATS_PER_CREATURE;
    let changed = count !== this.prevCount;

    if (!changed) {
      const prev = this.prev;
      for (let i = 0; i < floats; i++) {
        if (prev[i] !== buffer[i]) {
          changed = true;
          break;
        }
      }
    }

    if (changed) {
      if (this.prev.length < floats) {
        this.prev = new Float32Array(Math.max(floats, this.prev.length * 2));
      }
      this.prev.set(buffer.subarray(0, floats));
      this.prevCount = count;
    }
    return changed;
  }

  reset(): void {
    this.lastTick = -1;
    this.prevCount = -1;
  }
}
