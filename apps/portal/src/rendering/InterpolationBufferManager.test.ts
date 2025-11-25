import { describe, it, expect, beforeEach } from "vitest";
import { InterpolationBufferManager } from "./InterpolationBufferManager";
import type { CreatureData } from "@/types/GameState";

describe("InterpolationBufferManager", () => {
  let manager: InterpolationBufferManager;

  beforeEach(() => {
    manager = new InterpolationBufferManager();
  });

  describe("initialization", () => {
    it("should initialize with empty buffer", () => {
      expect(manager.getBuffer().length).toBe(0);
      expect(manager.getCreatureCount()).toBe(0);
    });

    it("should initialize with START = END for first frame", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 100, y: 50, rotation: 0, size: 10 },
      ];

      manager.initialize(creatures);

      const buffer = manager.getBuffer();
      expect(buffer.length).toBe(7); // 1 creature * 7 floats (no id in buffer)

      // START position
      expect(buffer[0]).toBe(100); // startX
      expect(buffer[1]).toBe(50); // startY

      // END position (same as START initially)
      expect(buffer[2]).toBe(100); // endX
      expect(buffer[3]).toBe(50); // endY

      // Rotation
      expect(buffer[4]).toBe(0); // startRot
      expect(buffer[5]).toBe(0); // endRot

      // Size
      expect(buffer[6]).toBe(10); // size
    });

    it("should initialize multiple creatures", () => {
      const creatures: CreatureData[] = [
        { id: 1, x: 100, y: 50, rotation: 0.5, size: 10 },
        { id: 2, x: 200, y: 75, rotation: 1.0, size: 12 },
      ];

      manager.initialize(creatures);

      const buffer = manager.getBuffer();
      expect(buffer.length).toBe(14); // 2 creatures * 7 floats

      // Creature 1 (offset 0)
      expect(buffer[0]).toBe(100); // startX
      expect(buffer[2]).toBe(100); // endX
      expect(buffer[6]).toBe(10); // size

      // Creature 2 (offset 7)
      expect(buffer[7]).toBe(200); // startX
      expect(buffer[9]).toBe(200); // endX
      expect(buffer[13]).toBe(12); // size
    });
  });

  describe("buffer swap on update", () => {
    it("should swap END → START on simulation tick", () => {
      // Initialize at position (0, 0)
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Update to position (100, 50)
      const newCreatures: CreatureData[] = [
        { id: 1, x: 100, y: 50, rotation: 1.5, size: 10 },
      ];
      manager.update(newCreatures);

      const buffer = manager.getBuffer();

      // START should be old END (0, 0)
      expect(buffer[0]).toBe(0); // startX
      expect(buffer[1]).toBe(0); // startY
      expect(buffer[4]).toBe(0); // startRot

      // END should be new position (100, 50)
      expect(buffer[2]).toBe(100); // endX
      expect(buffer[3]).toBe(50); // endY
      expect(buffer[5]).toBe(1.5); // endRot
    });

    it("should handle multiple updates correctly", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // First update: 0 → 50
      manager.update([{ id: 1, x: 50, y: 25, rotation: 0.5, size: 10 }]);

      let buffer = manager.getBuffer();
      expect(buffer[0]).toBe(0); // startX
      expect(buffer[2]).toBe(50); // endX

      // Second update: 50 → 100
      manager.update([{ id: 1, x: 100, y: 50, rotation: 1.0, size: 10 }]);

      buffer = manager.getBuffer();
      expect(buffer[0]).toBe(50); // startX (was previous endX)
      expect(buffer[2]).toBe(100); // endX (new)
    });

    it("should preserve size on update", () => {
      manager.initialize([{ id: 42, x: 0, y: 0, rotation: 0, size: 15 }]);
      manager.update([{ id: 42, x: 100, y: 50, rotation: 1.0, size: 15 }]);

      const buffer = manager.getBuffer();
      expect(buffer[6]).toBe(15); // size preserved
    });
  });

  describe("creature count changes", () => {
    it("should handle spawning new creatures", () => {
      // Start with 1 creature
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Spawn a second creature
      const newCreatures: CreatureData[] = [
        { id: 1, x: 10, y: 10, rotation: 0, size: 10 },
        { id: 2, x: 20, y: 20, rotation: 0.5, size: 12 },
      ];
      manager.update(newCreatures);

      const buffer = manager.getBuffer();
      expect(buffer.length).toBe(14); // 2 creatures * 7 floats
      expect(manager.getCreatureCount()).toBe(2);

      // Creature 2 should be initialized with START = END (just spawned, offset 7)
      expect(buffer[7]).toBe(20); // startX
      expect(buffer[9]).toBe(20); // endX (same as start)
    });

    it("should handle despawning creatures", () => {
      // Start with 2 creatures
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0, size: 10 },
      ]);

      // Despawn creature 2
      manager.update([{ id: 1, x: 10, y: 10, rotation: 0.5, size: 10 }]);

      const buffer = manager.getBuffer();
      expect(buffer.length).toBe(7); // 1 creature * 7 floats
      expect(manager.getCreatureCount()).toBe(1);
    });

    it("should handle complete population change", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Replace with entirely different creatures
      const newCreatures: CreatureData[] = [
        { id: 10, x: 50, y: 50, rotation: 1.0, size: 15 },
        { id: 11, x: 60, y: 60, rotation: 1.5, size: 20 },
        { id: 12, x: 70, y: 70, rotation: 2.0, size: 25 },
      ];
      manager.update(newCreatures);

      const buffer = manager.getBuffer();
      expect(buffer.length).toBe(21); // 3 creatures * 7 floats
      expect(manager.getCreatureCount()).toBe(3);
    });
  });

  describe("buffer resizing", () => {
    it("should resize buffer when creature count increases significantly", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Spawn many creatures
      const manyCreatures: CreatureData[] = [];
      for (let i = 0; i < 1000; i++) {
        manyCreatures.push({
          id: i,
          x: i * 10,
          y: i * 10,
          rotation: i * 0.1,
          size: 10,
        });
      }
      manager.update(manyCreatures);

      const buffer = manager.getBuffer();
      expect(buffer.length).toBe(7000); // 1000 creatures * 7 floats
      expect(manager.getCreatureCount()).toBe(1000);
    });

    it("should handle buffer shrinking", () => {
      // Start with many creatures
      const manyCreatures: CreatureData[] = [];
      for (let i = 0; i < 1000; i++) {
        manyCreatures.push({
          id: i,
          x: i * 10,
          y: i * 10,
          rotation: 0,
          size: 10,
        });
      }
      manager.initialize(manyCreatures);

      // Shrink to 10 creatures
      manager.update([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 10, y: 10, rotation: 0, size: 10 },
      ]);

      expect(manager.getCreatureCount()).toBe(2);
      // Buffer may stay allocated at larger size (optimization)
      // but logical length should be correct
    });
  });

  describe("direct buffer access", () => {
    it("should provide read-only access to buffer", () => {
      manager.initialize([{ id: 1, x: 100, y: 50, rotation: 0, size: 10 }]);

      const buffer = manager.getBuffer();
      expect(buffer).toBeInstanceOf(Float32Array);
      expect(buffer.length).toBe(7); // 7 floats per creature (no id)
    });

    it("should track buffer dirty state", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      expect(manager.isDirty()).toBe(true); // Dirty after init

      manager.markClean();
      expect(manager.isDirty()).toBe(false);

      manager.update([{ id: 1, x: 100, y: 50, rotation: 1.0, size: 10 }]);
      expect(manager.isDirty()).toBe(true); // Dirty after update
    });
  });

  describe("edge cases", () => {
    it("should handle empty creature list", () => {
      manager.initialize([]);

      expect(manager.getBuffer().length).toBe(0);
      expect(manager.getCreatureCount()).toBe(0);
    });

    it("should handle update with empty list after initialization", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);
      manager.update([]);

      expect(manager.getCreatureCount()).toBe(0);
      expect(manager.getBuffer().length).toBe(0);
    });

    it("should handle creatures with extreme values", () => {
      manager.initialize([
        {
          id: 999999,
          x: -10000,
          y: 10000,
          rotation: Math.PI * 2,
          size: 100,
        },
      ]);

      const buffer = manager.getBuffer();
      expect(buffer[0]).toBe(-10000); // startX
      expect(buffer[1]).toBe(10000); // startY
      expect(buffer[4]).toBeCloseTo(Math.PI * 2); // startRot
      expect(buffer[6]).toBe(100); // size
    });
  });

  describe("performance", () => {
    it("should handle 100K creatures efficiently", () => {
      const creatures: CreatureData[] = [];
      for (let i = 0; i < 100000; i++) {
        creatures.push({
          id: i,
          x: Math.random() * 1000,
          y: Math.random() * 1000,
          rotation: Math.random() * Math.PI * 2,
          size: 10,
        });
      }

      const startInit = performance.now();
      manager.initialize(creatures);
      const initTime = performance.now() - startInit;

      expect(initTime).toBeLessThan(50); // Should init in <50ms

      // Update with slight position changes
      const updatedCreatures = creatures.map((c) => ({
        ...c,
        x: c.x + 1,
        y: c.y + 1,
      }));

      const startUpdate = performance.now();
      manager.update(updatedCreatures);
      const updateTime = performance.now() - startUpdate;

      expect(updateTime).toBeLessThan(10); // Should update in <10ms
      expect(manager.getCreatureCount()).toBe(100000);
    });
  });
});
