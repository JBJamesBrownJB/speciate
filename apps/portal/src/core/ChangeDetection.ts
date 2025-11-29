import type { CreatureData } from "@/types/GameState";

/**
 * State tracker for detecting when renderer should update
 */
export class ChangeDetector {
  private lastCreatureCount = 0;
  private lastStateHash: string | null = null;

  /**
   * Check if state has changed and renderer should update
   * @param creatures - Current creature data from simulation
   * @returns true if renderer should update, false otherwise
   */
  shouldUpdate(creatures: CreatureData[]): boolean {
    const currentCount = creatures.length;

    // Always update if count changed (spawns/despawns)
    if (currentCount !== this.lastCreatureCount) {
      this.lastCreatureCount = currentCount;
      this.lastStateHash = this.computeHash(creatures);
      return true;
    }

    // If count is same, check if any positions changed
    const currentHash = this.computeHash(creatures);
    const hasChanged = currentHash !== this.lastStateHash;

    if (hasChanged) {
      this.lastStateHash = currentHash;
      return true;
    }

    return false;
  }

  /**
   * Compute lightweight hash of creature positions for change detection
   * Only checks first and last 3 creatures to avoid O(n) hash on every frame
   */
  private computeHash(creatures: CreatureData[]): string {
    if (creatures.length === 0) return "empty";

    // Sample first 3 and last 3 creatures
    const sampleSize = Math.min(3, creatures.length);
    let hash = "";

    // First 3
    for (let i = 0; i < sampleSize; i++) {
      const c = creatures[i];
      hash += `${c.id},${c.x.toFixed(2)},${c.y.toFixed(2)};`;
    }

    // Last 3 (if different from first 3)
    if (creatures.length > sampleSize) {
      for (let i = creatures.length - sampleSize; i < creatures.length; i++) {
        const c = creatures[i];
        hash += `${c.id},${c.x.toFixed(2)},${c.y.toFixed(2)};`;
      }
    }

    return hash;
  }

  /**
   * Reset detector state (e.g., when simulation restarts)
   */
  reset(): void {
    this.lastCreatureCount = 0;
    this.lastStateHash = null;
  }
}
