import type { CreatureData } from "@/types/GameState";

export class ChangeDetector {
  private lastCreatureCount = 0;
  private lastStateHash: string | null = null;

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

  reset(): void {
    this.lastCreatureCount = 0;
    this.lastStateHash = null;
  }
}
