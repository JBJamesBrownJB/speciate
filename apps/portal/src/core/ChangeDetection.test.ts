import { describe, it, expect, beforeEach } from "vitest";
import { ChangeDetector } from "./ChangeDetection";
import type { CreatureData } from "@/types/GameState";

function createCreature(id: number, x: number, y: number): CreatureData {
  return { id, x, y, rotation: 0, size: 1.0 };
}

describe("ChangeDetector", () => {
  let detector: ChangeDetector;

  beforeEach(() => {
    detector = new ChangeDetector();
  });

  describe("shouldUpdate - count changes", () => {
    it("should update when creatures are spawned (count increases)", () => {
      const initial: CreatureData[] = [];
      const afterSpawn = [createCreature(1, 0, 0)];

      expect(detector.shouldUpdate(initial)).toBe(true); // First call always true (count 0 → 0)
      expect(detector.shouldUpdate(afterSpawn)).toBe(true); // Count changed 0 → 1
    });

    it("should update when creatures are despawned (count decreases)", () => {
      const initial = [createCreature(1, 0, 0), createCreature(2, 10, 10)];
      const afterDespawn = [createCreature(1, 0, 0)];

      expect(detector.shouldUpdate(initial)).toBe(true);
      expect(detector.shouldUpdate(initial)).toBe(false); // No change
      expect(detector.shouldUpdate(afterDespawn)).toBe(true); // Count changed 2 → 1
    });

    it("should update when loading trial after clear (0 → many)", () => {
      const empty: CreatureData[] = [];
      const trialCreatures = Array.from({ length: 100 }, (_, i) =>
        createCreature(i, i * 2, i * 2)
      );

      expect(detector.shouldUpdate(empty)).toBe(true);
      expect(detector.shouldUpdate(empty)).toBe(false); // Still empty
      expect(detector.shouldUpdate(trialCreatures)).toBe(true); // 0 → 100
    });
  });

  describe("shouldUpdate - position changes", () => {
    it("should update when wandering creatures move (count same)", () => {
      const initial = [createCreature(1, 0, 0), createCreature(2, 10, 10)];
      const afterMove = [createCreature(1, 0.5, 0.3), createCreature(2, 10, 10)];

      expect(detector.shouldUpdate(initial)).toBe(true);
      expect(detector.shouldUpdate(initial)).toBe(false); // No change
      expect(detector.shouldUpdate(afterMove)).toBe(true); // Position changed
    });

    it("should update when ANY sampled creature moves", () => {
      // Create 100 creatures, only last one moves
      const initial = Array.from({ length: 100 }, (_, i) =>
        createCreature(i, i * 2, i * 2)
      );
      const afterMove = Array.from({ length: 100 }, (_, i) =>
        createCreature(i, i * 2, i === 99 ? 1000 : i * 2) // Last creature moves
      );

      expect(detector.shouldUpdate(initial)).toBe(true);
      expect(detector.shouldUpdate(initial)).toBe(false);
      expect(detector.shouldUpdate(afterMove)).toBe(true); // Last creature in sample moved
    });

    it("should NOT update if middle creatures move (not sampled)", () => {
      // Only first 3 and last 3 are sampled
      const initial = Array.from({ length: 100 }, (_, i) =>
        createCreature(i, i * 2, i * 2)
      );
      const afterMove = Array.from({ length: 100 }, (_, i) =>
        createCreature(i, i === 50 ? 1000 : i * 2, i * 2) // Middle creature moves
      );

      expect(detector.shouldUpdate(initial)).toBe(true);
      expect(detector.shouldUpdate(afterMove)).toBe(false); // Middle not sampled
    });
  });

  describe("shouldUpdate - stationary creatures (catatonic)", () => {
    it("should NOT update if catatonic creatures don't move", () => {
      const catatonics = [createCreature(1, 10, 10), createCreature(2, 20, 20)];

      expect(detector.shouldUpdate(catatonics)).toBe(true); // First call
      expect(detector.shouldUpdate(catatonics)).toBe(false); // No change
      expect(detector.shouldUpdate(catatonics)).toBe(false); // Still no change
    });

    it("should update when new creature spawns among catatonics", () => {
      const catatonics = [createCreature(1, 10, 10), createCreature(2, 20, 20)];
      const withNew = [
        createCreature(1, 10, 10),
        createCreature(2, 20, 20),
        createCreature(3, 30, 30),
      ];

      expect(detector.shouldUpdate(catatonics)).toBe(true);
      expect(detector.shouldUpdate(catatonics)).toBe(false);
      expect(detector.shouldUpdate(withNew)).toBe(true); // Count changed
    });
  });

  describe("shouldUpdate - mixed scenarios", () => {
    it("should handle catatonic crowd + wandering seeker", () => {
      // 99 catatonics + 1 seeker (last position)
      const catatonics = Array.from({ length: 99 }, (_, i) =>
        createCreature(i, i * 2, i * 2)
      );
      const initial = [...catatonics, createCreature(99, 0, 0)]; // Seeker at end

      const afterSeek = [...catatonics, createCreature(99, 10, 5)]; // Seeker moved

      expect(detector.shouldUpdate(initial)).toBe(true);
      expect(detector.shouldUpdate(initial)).toBe(false);
      expect(detector.shouldUpdate(afterSeek)).toBe(true); // Seeker (sampled) moved
    });

    it("should handle crowd-navigation trial scenario", () => {
      // 100 catatonics (grid) + 1 seeker
      const grid = Array.from({ length: 100 }, (_, i) =>
        createCreature(i, (i % 10) * 2, Math.floor(i / 10) * 2)
      );
      const initialWithSeeker = [...grid, createCreature(100, -30, 0)];

      // Seeker moves toward target
      const afterTick1 = [...grid, createCreature(100, -29, 0)];
      const afterTick2 = [...grid, createCreature(100, -28, 0)];

      expect(detector.shouldUpdate(initialWithSeeker)).toBe(true);
      expect(detector.shouldUpdate(afterTick1)).toBe(true); // Seeker moved (last sampled)
      expect(detector.shouldUpdate(afterTick2)).toBe(true); // Seeker moved again
    });
  });

  describe("reset", () => {
    it("should reset detector state", () => {
      const creatures = [createCreature(1, 0, 0)];

      expect(detector.shouldUpdate(creatures)).toBe(true);
      expect(detector.shouldUpdate(creatures)).toBe(false);

      detector.reset();

      expect(detector.shouldUpdate(creatures)).toBe(true); // After reset, treats as new
    });
  });

  describe("edge cases", () => {
    it("should handle empty arrays", () => {
      const empty: CreatureData[] = [];

      expect(detector.shouldUpdate(empty)).toBe(true); // First call
      expect(detector.shouldUpdate(empty)).toBe(false); // Still empty
    });

    it("should handle single creature", () => {
      const single = [createCreature(1, 0, 0)];
      const moved = [createCreature(1, 5, 5)];

      expect(detector.shouldUpdate(single)).toBe(true);
      expect(detector.shouldUpdate(moved)).toBe(true); // Position changed
    });

    it("should handle very small position changes", () => {
      const initial = [createCreature(1, 0, 0)];
      const microMove = [createCreature(1, 0.001, 0.001)]; // Sub-pixel

      expect(detector.shouldUpdate(initial)).toBe(true);
      expect(detector.shouldUpdate(microMove)).toBe(false); // Rounded to same value
    });
  });
});
