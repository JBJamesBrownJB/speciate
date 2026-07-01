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

  describe("push-on-swap path (tick > 0): tick identity is the change signal", () => {
    const creatures = [createCreature(1, 0, 0)];

    it("first delivery always updates", () => {
      expect(detector.shouldUpdate(creatures, 1)).toBe(true);
    });

    it("a new tick is new data — even if positions look identical", () => {
      expect(detector.shouldUpdate(creatures, 1)).toBe(true);
      expect(detector.shouldUpdate(creatures, 2)).toBe(true);
    });

    it("the same tick delivered twice is a duplicate", () => {
      expect(detector.shouldUpdate(creatures, 5)).toBe(true);
      expect(detector.shouldUpdate(creatures, 5)).toBe(false);
    });
  });

  describe("poll-fallback path (tick 0): exact comparison, no sampling", () => {
    it("updates when creatures are spawned (count increases)", () => {
      expect(detector.shouldUpdate([], 0)).toBe(true); // first call
      expect(detector.shouldUpdate([createCreature(1, 0, 0)], 0)).toBe(true);
    });

    it("updates when creatures are despawned (count decreases)", () => {
      const initial = [createCreature(1, 0, 0), createCreature(2, 10, 10)];
      expect(detector.shouldUpdate(initial, 0)).toBe(true);
      expect(detector.shouldUpdate(initial, 0)).toBe(false);
      expect(detector.shouldUpdate([initial[0]], 0)).toBe(true);
    });

    it("updates when ONLY a middle creature moves (the old sampled hash missed this)", () => {
      // Catatonic creatures at the edges, one mover in the middle — the exact
      // failure mode of the first-3/last-3 sample: a visibly frozen mover.
      const initial = Array.from({ length: 100 }, (_, i) => createCreature(i, i * 2, i * 2));
      const afterMove = initial.map((c, i) => (i === 50 ? createCreature(50, 1000, 100) : c));

      expect(detector.shouldUpdate(initial, 0)).toBe(true);
      expect(detector.shouldUpdate(afterMove, 0)).toBe(true);
    });

    it("does NOT update when nothing moved (catatonic crowd)", () => {
      const catatonics = [createCreature(1, 10, 10), createCreature(2, 20, 20)];
      expect(detector.shouldUpdate(catatonics, 0)).toBe(true);
      expect(detector.shouldUpdate(catatonics, 0)).toBe(false);
      expect(detector.shouldUpdate(catatonics, 0)).toBe(false);
    });

    it("detects rotation-only changes (spinning in place)", () => {
      const initial = [createCreature(1, 5, 5)];
      const spun = [{ ...createCreature(1, 5, 5), rotation: 1.5 }];
      expect(detector.shouldUpdate(initial, 0)).toBe(true);
      expect(detector.shouldUpdate(spun, 0)).toBe(true);
    });

    it("detects id changes at identical positions (death + respawn)", () => {
      const before = [createCreature(1, 5, 5)];
      const respawned = [createCreature(2, 5, 5)];
      expect(detector.shouldUpdate(before, 0)).toBe(true);
      expect(detector.shouldUpdate(respawned, 0)).toBe(true);
    });

    it("detects even sub-pixel moves — exact means exact", () => {
      expect(detector.shouldUpdate([createCreature(1, 0, 0)], 0)).toBe(true);
      expect(detector.shouldUpdate([createCreature(1, 0.001, 0.001)], 0)).toBe(true);
    });

    it("compares by value, not reference — safe with the reused object pool", () => {
      const pooled = [createCreature(1, 0, 0)];
      expect(detector.shouldUpdate(pooled, 0)).toBe(true);
      expect(detector.shouldUpdate(pooled, 0)).toBe(false);
      pooled[0].x = 99; // IPC client mutates pooled objects in place
      expect(detector.shouldUpdate(pooled, 0)).toBe(true);
    });

    it("handles empty arrays", () => {
      expect(detector.shouldUpdate([], 0)).toBe(true);
      expect(detector.shouldUpdate([], 0)).toBe(false);
    });
  });

  describe("reset", () => {
    it("resets both paths", () => {
      const creatures = [createCreature(1, 0, 0)];
      expect(detector.shouldUpdate(creatures, 3)).toBe(true);
      detector.reset();
      expect(detector.shouldUpdate(creatures, 3)).toBe(true);

      expect(detector.shouldUpdate(creatures, 0)).toBe(true);
      expect(detector.shouldUpdate(creatures, 0)).toBe(false);
      detector.reset();
      expect(detector.shouldUpdate(creatures, 0)).toBe(true);
    });
  });
});
