import { Creature } from "./Creature";
import { RENDERING_CONFIG } from "../core/constants";

/**
 * Stores old and new states for interpolation
 */
interface CreatureSnapshot {
  old: Creature | null;
  new: Creature;
  oldTime: number;
  newTime: number;
}

/**
 * Interpolator smooths creature movement between server updates.
 *
 * The backend sends updates at variable rate (typically 20 Hz), but we render at 60+ FPS.
 * This class stores the last two positions for each creature and interpolates
 * between them to create smooth movement.
 *
 * The interpolation buffer is ADAPTIVE - it automatically measures the server's
 * update rate and calculates the optimal buffer size. This eliminates coupling
 * between portal and simulation tick rates.
 *
 * IMPORTANT - JITTER WARNING:
 * Visual jitter/stuttering occurs if the adaptive buffer multiplier doesn't match
 * the server's tick rate rhythm. Current multiplier: 1.0× (optimal for smooth motion).
 *
 * - 1.0× (current): Minimal lag, matches update interval perfectly - SMOOTH ✓
 * - 1.5×: Extra 50% lag - causes visible stuttering/sluggishness
 * - <1.0×: Insufficient buffer - jerky interpolation
 *
 * Diagnosis: Check getBufferMs() ≈ median update interval (~50ms for 20 Hz).
 */
export class Interpolator {
  private snapshots: Map<number, CreatureSnapshot> = new Map();

  // Adaptive buffer calculation
  private updateIntervals: number[] = [];
  private lastUpdateTime: number = 0;
  private readonly INTERVAL_HISTORY_SIZE = 10;

  /**
   * Update with new creature data from the server
   * @param creatures Latest creature positions
   * @param timestamp Time of this update (milliseconds)
   */
  update(creatures: Creature[], timestamp: number): void {
    // Track update intervals for adaptive buffer calculation
    if (this.lastUpdateTime > 0) {
      const interval = timestamp - this.lastUpdateTime;
      this.updateIntervals.push(interval);
      if (this.updateIntervals.length > this.INTERVAL_HISTORY_SIZE) {
        this.updateIntervals.shift();
      }
    }
    this.lastUpdateTime = timestamp;

    const currentIds = new Set<number>();

    for (const creature of creatures) {
      currentIds.add(creature.id);

      const existing = this.snapshots.get(creature.id);

      if (existing) {
        // Move 'new' to 'old' and store the latest as 'new'
        this.snapshots.set(creature.id, {
          old: existing.new,
          new: creature,
          oldTime: existing.newTime,
          newTime: timestamp,
        });
      } else {
        // First time seeing this creature
        this.snapshots.set(creature.id, {
          old: null,
          new: creature,
          oldTime: timestamp,
          newTime: timestamp,
        });
      }
    }

    // Remove creatures that are no longer in the update
    for (const id of this.snapshots.keys()) {
      if (!currentIds.has(id)) {
        this.snapshots.delete(id);
      }
    }
  }

  /**
   * Get interpolated creature positions for rendering
   * @param currentTime Current time (milliseconds)
   * @returns Array of interpolated creatures
   */
  interpolate(currentTime: number): Creature[] {
    // Apply adaptive buffer - render in the past to ensure interpolation
    const buffer = this.getAdaptiveBuffer();
    const renderTime = currentTime - buffer;

    const result: Creature[] = [];

    for (const snapshot of this.snapshots.values()) {
      if (!snapshot.old) {
        // No old position yet, use latest
        result.push(snapshot.new);
        continue;
      }

      // Calculate interpolation alpha (0 to 1)
      const duration = snapshot.newTime - snapshot.oldTime;
      const elapsed = renderTime - snapshot.oldTime;

      let alpha = duration > 0 ? elapsed / duration : 1;

      // If we're past the latest snapshot and have velocity data, extrapolate with damping
      if (
        alpha > 1 &&
        snapshot.new.vx !== undefined &&
        snapshot.new.vy !== undefined
      ) {
        const dt = (renderTime - snapshot.newTime) / 1000; // seconds
        const damping = Math.exp(-RENDERING_CONFIG.VELOCITY_DAMPING * dt);

        const x = snapshot.new.x + snapshot.new.vx * dt * damping;
        const y = snapshot.new.y + snapshot.new.vy * dt * damping;
        const rotation = snapshot.new.rotation; // Don't extrapolate rotation

        result.push(snapshot.new.withTransform(x, y, rotation));
        continue;
      }

      // Clamp alpha for interpolation
      alpha = Math.max(0, Math.min(1, alpha));

      // Interpolate position
      const x = this.lerp(snapshot.old.x, snapshot.new.x, alpha);
      const y = this.lerp(snapshot.old.y, snapshot.new.y, alpha);

      // Interpolate rotation (handling wraparound)
      const rotation = this.lerpAngle(
        snapshot.old.rotation,
        snapshot.new.rotation,
        alpha
      );

      // Create interpolated creature
      result.push(snapshot.new.withTransform(x, y, rotation));
    }

    return result;
  }

  /**
   * Remove all creatures
   */
  clear(): void {
    this.snapshots.clear();
  }

  /**
   * Get number of tracked creatures
   */
  getCreatureCount(): number {
    return this.snapshots.size;
  }

  /**
   * Calculate adaptive interpolation buffer based on observed update rate.
   *
   * This measures the actual time between server updates and calculates
   * an optimal buffer size to ensure smooth interpolation. The buffer is
   * set to 1.0× the median update interval for minimal lag while ensuring
   * we always have at least 2 snapshots available for interpolation.
   *
   * @returns Buffer size in milliseconds
   */
  private getAdaptiveBuffer(): number {
    // Need at least 3 intervals to calculate reliable median
    if (this.updateIntervals.length < 3) {
      return RENDERING_CONFIG.INTERPOLATION_BUFFER_MS; // Fallback
    }

    // Calculate median (more robust to outliers than mean)
    const sorted = [...this.updateIntervals].sort((a, b) => a - b);
    const median = sorted[Math.floor(sorted.length / 2)];

    // Buffer = 1.0× median interval (minimal lag, optimal for smooth interpolation)
    // Clamp between 20ms and 200ms for sanity (handles extreme cases)
    const buffer = median;
    return Math.max(20, Math.min(200, buffer));
  }

  /**
   * Get current adaptive buffer size (for debugging/monitoring)
   * @returns Current buffer in milliseconds
   */
  getBufferMs(): number {
    return this.getAdaptiveBuffer();
  }

  /**
   * Linear interpolation
   */
  private lerp(start: number, end: number, alpha: number): number {
    return start + (end - start) * alpha;
  }

  /**
   * Angular interpolation (handles 2π wraparound)
   */
  private lerpAngle(start: number, end: number, alpha: number): number {
    // Calculate shortest path between angles
    let diff = end - start;

    // Normalize to [-π, π]
    while (diff > Math.PI) diff -= 2 * Math.PI;
    while (diff < -Math.PI) diff += 2 * Math.PI;

    return start + diff * alpha;
  }
}
