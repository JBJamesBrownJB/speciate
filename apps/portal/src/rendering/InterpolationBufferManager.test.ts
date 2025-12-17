import { describe, it, expect, beforeEach } from "vitest";
import { InterpolationBufferManager } from "./InterpolationBufferManager";
import type { CreatureData } from "@/types/GameState";

describe("InterpolationBufferManager", () => {
  let manager: InterpolationBufferManager;

  beforeEach(() => {
    manager = new InterpolationBufferManager(1000); // Small capacity for tests
  });

  describe("initialization", () => {
    it("should initialize with pre-allocated capacity", () => {
      expect(manager.getBuffer().length).toBe(0); // No creatures yet
      expect(manager.getCreatureCount()).toBe(0);
      expect(manager.getCapacity()).toBe(1000); // Pre-allocated
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

  describe("buffer capacity and reuse", () => {
    it("should reuse buffer when spawning within capacity (no allocation)", () => {
      const smallManager = new InterpolationBufferManager(100);
      const initialCapacity = smallManager.getCapacity();

      smallManager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Spawn more creatures (still within capacity)
      for (let count = 10; count <= 90; count += 10) {
        const creatures: CreatureData[] = [];
        for (let i = 0; i < count; i++) {
          creatures.push({ id: i, x: i, y: i, rotation: 0, size: 10 });
        }
        smallManager.update(creatures);

        // Capacity should NOT change - buffer reused
        expect(smallManager.getCapacity()).toBe(initialCapacity);
        expect(smallManager.getCreatureCount()).toBe(count);
      }
    });

    it("should grow capacity when exceeding initial capacity", () => {
      const smallManager = new InterpolationBufferManager(10);
      expect(smallManager.getCapacity()).toBe(10);

      // Spawn more than capacity
      const creatures: CreatureData[] = [];
      for (let i = 0; i < 25; i++) {
        creatures.push({ id: i, x: i, y: i, rotation: 0, size: 10 });
      }
      smallManager.initialize(creatures);

      // Capacity should have grown (doubled or to fit)
      expect(smallManager.getCapacity()).toBeGreaterThanOrEqual(25);
      expect(smallManager.getCreatureCount()).toBe(25);
    });

    it("should handle buffer shrinking without reallocating", () => {
      const smallManager = new InterpolationBufferManager(100);

      // Initialize with many creatures
      const manyCreatures: CreatureData[] = [];
      for (let i = 0; i < 50; i++) {
        manyCreatures.push({ id: i, x: i, y: i, rotation: 0, size: 10 });
      }
      smallManager.initialize(manyCreatures);
      const capacityAfterInit = smallManager.getCapacity();

      // Shrink to fewer creatures
      smallManager.update([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 10, y: 10, rotation: 0, size: 10 },
      ]);

      // Capacity stays same (no shrink reallocation)
      expect(smallManager.getCapacity()).toBe(capacityAfterInit);
      expect(smallManager.getCreatureCount()).toBe(2);
      expect(smallManager.getBuffer().length).toBe(14); // 2 * 7 floats
    });

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

      expect(initTime).toBeLessThan(150); // Should init in <150ms

      // Update with slight position changes
      const updatedCreatures = creatures.map((c) => ({
        ...c,
        x: c.x + 1,
        y: c.y + 1,
      }));

      const startUpdate = performance.now();
      manager.update(updatedCreatures);
      const updateTime = performance.now() - startUpdate;

      expect(updateTime).toBeLessThan(150); // Should update in <150ms (Map lookup overhead, CI variance)
      expect(manager.getCreatureCount()).toBe(100000);
    });
  });

  describe("ID-based tracking (viewport culling support)", () => {
    it("should maintain interpolation state when creature order changes", () => {
      // Initialize with creatures in order [1, 2, 3]
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0.5, size: 10 },
        { id: 3, x: 200, y: 200, rotation: 1.0, size: 10 },
      ]);

      // Update with reversed order [3, 2, 1] but same IDs
      manager.update([
        { id: 3, x: 210, y: 210, rotation: 1.1, size: 10 },
        { id: 2, x: 110, y: 110, rotation: 0.6, size: 10 },
        { id: 1, x: 10, y: 10, rotation: 0.1, size: 10 },
      ]);

      // Buffer output order should match input order (3, 2, 1)
      // Creature 3: was at (200,200), now at (210,210)
      const buffer = manager.getBuffer();

      // Creature 3 is now at index 0 (first in new order)
      expect(buffer[0]).toBe(200); // startX = old endX
      expect(buffer[1]).toBe(200); // startY = old endY
      expect(buffer[2]).toBe(210); // endX = new position
      expect(buffer[3]).toBe(210); // endY = new position

      // Creature 2 is now at index 1 (offset 7)
      expect(buffer[7]).toBe(100); // startX = old endX
      expect(buffer[9]).toBe(110); // endX = new position

      // Creature 1 is now at index 2 (offset 14)
      expect(buffer[14]).toBe(0); // startX = old endX
      expect(buffer[16]).toBe(10); // endX = new position
    });

    it("should initialize new creatures with START=END when entering viewport", () => {
      // Start with creature 1 only
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Update: creature 2 enters viewport (wasn't in previous tick)
      manager.update([
        { id: 1, x: 10, y: 10, rotation: 0.1, size: 10 },
        { id: 2, x: 500, y: 500, rotation: 1.0, size: 12 }, // NEW
      ]);

      const buffer = manager.getBuffer();

      // Creature 1 (index 0): normal interpolation
      expect(buffer[0]).toBe(0); // startX = old position
      expect(buffer[2]).toBe(10); // endX = new position

      // Creature 2 (index 1, offset 7): START=END (just entered)
      expect(buffer[7]).toBe(500); // startX = new position
      expect(buffer[8]).toBe(500); // startY = new position
      expect(buffer[9]).toBe(500); // endX = same as start
      expect(buffer[10]).toBe(500); // endY = same as start
      expect(buffer[11]).toBe(1.0); // startRot = new rotation
      expect(buffer[12]).toBe(1.0); // endRot = same as start
    });

    it("should cleanly remove creatures that leave viewport", () => {
      // Start with creatures 1, 2, 3
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0.5, size: 10 },
        { id: 3, x: 200, y: 200, rotation: 1.0, size: 10 },
      ]);

      // Update: creature 2 leaves (not in new data)
      manager.update([
        { id: 1, x: 10, y: 10, rotation: 0.1, size: 10 },
        { id: 3, x: 210, y: 210, rotation: 1.1, size: 10 },
      ]);

      expect(manager.getCreatureCount()).toBe(2);
      const buffer = manager.getBuffer();

      // Creature 1 at index 0
      expect(buffer[0]).toBe(0); // startX
      expect(buffer[2]).toBe(10); // endX

      // Creature 3 at index 1 (offset 7)
      expect(buffer[7]).toBe(200); // startX = old position
      expect(buffer[9]).toBe(210); // endX = new position
    });

    it("should NOT interpolate from stale cache when creature re-enters viewport", () => {
      // Start with creatures 1, 2
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0.5, size: 10 },
      ]);

      // Creature 2 leaves viewport (not visible this tick)
      manager.update([{ id: 1, x: 10, y: 10, rotation: 0.1, size: 10 }]);

      expect(manager.getCreatureCount()).toBe(1);

      // Creature 2 re-enters at a different position
      // Since creature 2 was NOT visible last tick, it should NOT interpolate from stale cache
      manager.update([
        { id: 1, x: 20, y: 20, rotation: 0.2, size: 10 },
        { id: 2, x: 500, y: 500, rotation: 2.0, size: 10 }, // RE-ENTERED
      ]);

      const buffer = manager.getBuffer();

      // Creature 1 at index 0: normal interpolation continues
      expect(buffer[0]).toBe(10); // startX = previous endX
      expect(buffer[2]).toBe(20); // endX = new position

      // Creature 2 at index 1: START=END (was not visible last tick, avoid ghosting)
      expect(buffer[7]).toBe(500); // startX = new position (not stale 100!)
      expect(buffer[9]).toBe(500); // endX = same
    });

it("should use START=END for creatures re-entering viewport (ghosting fix)", () => {
      // This test verifies the fix for the ghosting bug:
      // When creatures temporarily leave viewport and re-enter, they should
      // use START=END to avoid interpolating from stale cached position

      // Initialize with creatures 1 and 2
      manager.initialize([
        { id: 1, x: 10, y: 20, rotation: 0, size: 10 },
        { id: 2, x: 30, y: 40, rotation: 0.5, size: 10 },
      ]);

      // Creature 2 exits viewport (only creature 1 in update)
      manager.update([{ id: 1, x: 15, y: 25, rotation: 0.1, size: 10 }]);

      expect(manager.getCreatureCount()).toBe(1);

      // Creature 2 re-enters viewport at a NEW position
      manager.update([
        { id: 1, x: 20, y: 30, rotation: 0.2, size: 10 },
        { id: 2, x: 35, y: 45, rotation: 0.6, size: 10 }, // RE-ENTERED
      ]);

      const buffer = manager.getBuffer();

      // Creature 1 (index 0): normal interpolation (was visible last tick)
      expect(buffer[0]).toBe(15); // startX = previous endX
      expect(buffer[2]).toBe(20); // endX = new position

      // Creature 2 (index 1, offset 7): START=END (was NOT visible last tick)
      // This prevents ghosting from stale cached position
      expect(buffer[7]).toBe(35); // startX = new position (not stale 30!)
      expect(buffer[8]).toBe(45); // startY = new position (not stale 40!)
      expect(buffer[9]).toBe(35); // endX = same
      expect(buffer[10]).toBe(45); // endY = same
    });

    it("should handle mixed scenario: some enter, some leave, some stay", () => {
      // Start with creatures 1, 2, 3
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0.5, size: 10 },
        { id: 3, x: 200, y: 200, rotation: 1.0, size: 10 },
      ]);

      // Mixed update:
      // - Creature 1 stays
      // - Creature 2 leaves
      // - Creature 3 stays
      // - Creature 4 enters (new)
      manager.update([
        { id: 1, x: 10, y: 10, rotation: 0.1, size: 10 },
        { id: 3, x: 210, y: 210, rotation: 1.1, size: 10 },
        { id: 4, x: 300, y: 300, rotation: 1.5, size: 15 }, // NEW
      ]);

      expect(manager.getCreatureCount()).toBe(3);
      const buffer = manager.getBuffer();

      // Creature 1 at index 0: normal interpolation
      expect(buffer[0]).toBe(0); // startX = old
      expect(buffer[2]).toBe(10); // endX = new

      // Creature 3 at index 1: normal interpolation
      expect(buffer[7]).toBe(200); // startX = old
      expect(buffer[9]).toBe(210); // endX = new

      // Creature 4 at index 2: START=END (new)
      expect(buffer[14]).toBe(300); // startX = new
      expect(buffer[16]).toBe(300); // endX = same as start
    });
  });

  describe("rapid viewport oscillation (ghosting edge cases)", () => {
    it("should handle creature visible→gone→visible in 3 consecutive ticks", () => {
      // Tick 1: Creature visible
      manager.initialize([
        { id: 1, x: 100, y: 100, rotation: 0, size: 10 },
      ]);

      // Tick 2: Creature gone (culled)
      manager.update([]);
      expect(manager.getCreatureCount()).toBe(0);

      // Tick 3: Creature visible again at new position
      manager.update([
        { id: 1, x: 200, y: 200, rotation: 0.5, size: 10 },
      ]);

      const buffer = manager.getBuffer();
      // Should use START=END since creature wasn't visible last tick
      expect(buffer[0]).toBe(200); // startX = new position (not stale 100!)
      expect(buffer[2]).toBe(200); // endX = same
    });

    it("should handle alternating visibility for 10 ticks", () => {
      // Tick 0 (initialize): visible at (0, 0)
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
      ]);

      // Ticks 1-10: alternating visibility
      for (let tick = 1; tick <= 10; tick++) {
        const visible = tick % 2 === 0; // Even ticks visible (2,4,6,8,10), odd ticks gone
        const pos = tick * 10;

        if (visible) {
          manager.update([{ id: 1, x: pos, y: pos, rotation: 0, size: 10 }]);
          const buffer = manager.getBuffer();

          // Every time creature reappears after being gone, it should use START=END
          // (because it was gone the previous tick)
          expect(buffer[0]).toBe(pos); // startX = current pos
          expect(buffer[2]).toBe(pos); // endX = current pos
        } else {
          manager.update([]);
          expect(manager.getCreatureCount()).toBe(0);
        }
      }
    });

    it("should handle multiple creatures oscillating independently", () => {
      // Initialize both creatures
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 100, y: 100, rotation: 0, size: 10 },
      ]);

      // Tick 2: Only creature 1 visible
      manager.update([{ id: 1, x: 10, y: 10, rotation: 0, size: 10 }]);
      let buffer = manager.getBuffer();
      expect(manager.getCreatureCount()).toBe(1);
      expect(buffer[0]).toBe(0); // Creature 1: startX = old
      expect(buffer[2]).toBe(10); // Creature 1: endX = new

      // Tick 3: Only creature 2 visible (creature 1 now gone)
      manager.update([{ id: 2, x: 110, y: 110, rotation: 0, size: 10 }]);
      buffer = manager.getBuffer();
      expect(manager.getCreatureCount()).toBe(1);
      // Creature 2 wasn't visible last tick, so START=END
      expect(buffer[0]).toBe(110); // startX = new (not stale 100!)
      expect(buffer[2]).toBe(110); // endX = same

      // Tick 4: Both visible again
      manager.update([
        { id: 1, x: 20, y: 20, rotation: 0, size: 10 },
        { id: 2, x: 120, y: 120, rotation: 0, size: 10 },
      ]);
      buffer = manager.getBuffer();
      expect(manager.getCreatureCount()).toBe(2);
      // Creature 1: wasn't visible last tick → START=END
      expect(buffer[0]).toBe(20); // startX = new
      expect(buffer[2]).toBe(20); // endX = same
      // Creature 2: WAS visible last tick → normal interpolation
      expect(buffer[7]).toBe(110); // startX = previous endX
      expect(buffer[9]).toBe(120); // endX = new
    });

    it("should handle creature staying visible for multiple ticks then leaving and returning", () => {
      // Tick 1: Initialize at position 10
      manager.initialize([{ id: 1, x: 10, y: 10, rotation: 0, size: 10 }]);

      // Ticks 2-5: Creature stays visible, normal interpolation
      for (let tick = 2; tick <= 5; tick++) {
        const pos = tick * 10;
        manager.update([{ id: 1, x: pos, y: pos, rotation: 0, size: 10 }]);
        const buffer = manager.getBuffer();
        expect(buffer[0]).toBe((tick - 1) * 10); // startX = previous tick's pos
        expect(buffer[2]).toBe(pos); // endX = current pos
      }

      // Tick 6: Creature leaves
      manager.update([]);
      expect(manager.getCreatureCount()).toBe(0);

      // Tick 7: Creature returns at position 1000
      manager.update([{ id: 1, x: 1000, y: 1000, rotation: 0, size: 10 }]);
      const buffer = manager.getBuffer();
      // Should NOT interpolate from tick 5's position (50, 50)!
      expect(buffer[0]).toBe(1000); // startX = new (not stale 50!)
      expect(buffer[2]).toBe(1000); // endX = same
    });
  });

  describe("ID edge cases", () => {
    it("should handle duplicate IDs in same update (last position wins)", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Update with same ID appearing twice (shouldn't happen but test defense)
      manager.update([
        { id: 1, x: 100, y: 100, rotation: 0, size: 10 },
        { id: 1, x: 200, y: 200, rotation: 0, size: 10 }, // Same ID!
      ]);

      // Buffer should have 2 entries (input order preserved)
      // But stateById will only have last position
      expect(manager.getCreatureCount()).toBe(2);
    });

    it("should handle ID reuse - new creature spawns with recently dead creature's ID", () => {
      // Creature 1 at position (100, 100)
      manager.initialize([{ id: 1, x: 100, y: 100, rotation: 0, size: 10 }]);

      // Creature 1 dies (gone from update)
      manager.update([]);
      expect(manager.getCreatureCount()).toBe(0);

      // New creature spawns with same ID at completely different position
      manager.update([{ id: 1, x: 5000, y: 5000, rotation: 0, size: 10 }]);

      const buffer = manager.getBuffer();
      // Must NOT interpolate from dead creature's position!
      expect(buffer[0]).toBe(5000); // startX = new (not stale 100!)
      expect(buffer[2]).toBe(5000); // endX = same
    });

    it("should handle non-sequential IDs with large gaps", () => {
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 1000000, x: 100, y: 100, rotation: 0, size: 10 },
        { id: 5, x: 200, y: 200, rotation: 0, size: 10 },
      ]);

      manager.update([
        { id: 1000000, x: 110, y: 110, rotation: 0, size: 10 },
        { id: 1, x: 10, y: 10, rotation: 0, size: 10 },
        { id: 5, x: 210, y: 210, rotation: 0, size: 10 },
      ]);

      const buffer = manager.getBuffer();
      expect(manager.getCreatureCount()).toBe(3);

      // All should interpolate normally (all were visible last tick)
      expect(buffer[0]).toBe(100); // ID 1000000: startX = old
      expect(buffer[2]).toBe(110); // ID 1000000: endX = new
      expect(buffer[7]).toBe(0); // ID 1: startX = old
      expect(buffer[9]).toBe(10); // ID 1: endX = new
    });

    it("should handle very large IDs near MAX_SAFE_INTEGER", () => {
      const bigId = Number.MAX_SAFE_INTEGER - 1;

      manager.initialize([{ id: bigId, x: 0, y: 0, rotation: 0, size: 10 }]);
      manager.update([{ id: bigId, x: 100, y: 100, rotation: 0, size: 10 }]);

      const buffer = manager.getBuffer();
      expect(buffer[0]).toBe(0); // startX = old
      expect(buffer[2]).toBe(100); // endX = new
    });
  });

  describe("value edge cases", () => {
    it("should handle large position jumps while continuously visible", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      // Huge jump while staying visible (legitimate - fast creature)
      manager.update([{ id: 1, x: 10000, y: 10000, rotation: 0, size: 10 }]);

      const buffer = manager.getBuffer();
      // Should interpolate normally (creature was visible last tick)
      expect(buffer[0]).toBe(0); // startX = old
      expect(buffer[2]).toBe(10000); // endX = new (large jump OK)
    });

    it("should handle rotation wraparound (2π → small value)", () => {
      const nearTwoPi = Math.PI * 2 - 0.1;
      const smallAngle = 0.1;

      manager.initialize([{ id: 1, x: 0, y: 0, rotation: nearTwoPi, size: 10 }]);
      manager.update([{ id: 1, x: 0, y: 0, rotation: smallAngle, size: 10 }]);

      const buffer = manager.getBuffer();
      expect(buffer[4]).toBeCloseTo(nearTwoPi); // startRot
      expect(buffer[5]).toBeCloseTo(smallAngle); // endRot
      // Note: Shader may need to handle wraparound for smooth animation
    });

    it("should handle negative coordinates", () => {
      manager.initialize([{ id: 1, x: -1000, y: -2000, rotation: 0, size: 10 }]);
      manager.update([{ id: 1, x: -500, y: -1000, rotation: 0, size: 10 }]);

      const buffer = manager.getBuffer();
      expect(buffer[0]).toBe(-1000); // startX
      expect(buffer[2]).toBe(-500); // endX
    });

    it("should handle zero-size creatures", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 0 }]);

      const buffer = manager.getBuffer();
      expect(buffer[6]).toBe(0); // size = 0
    });
  });

  describe("state consistency invariants", () => {
    it("should maintain buffer length = creatureCount * 7 after complex operations", () => {
      // Series of complex operations
      manager.initialize([
        { id: 1, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 2, x: 0, y: 0, rotation: 0, size: 10 },
      ]);
      expect(manager.getBuffer().length).toBe(14);

      manager.update([{ id: 1, x: 10, y: 10, rotation: 0, size: 10 }]);
      expect(manager.getBuffer().length).toBe(7);

      manager.update([]);
      expect(manager.getBuffer().length).toBe(0);

      manager.update([
        { id: 5, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 6, x: 0, y: 0, rotation: 0, size: 10 },
        { id: 7, x: 0, y: 0, rotation: 0, size: 10 },
      ]);
      expect(manager.getBuffer().length).toBe(21);
    });

    it("should handle 100 rapid enter/exit cycles without state corruption", () => {
      manager.initialize([{ id: 1, x: 0, y: 0, rotation: 0, size: 10 }]);

      for (let i = 0; i < 100; i++) {
        // Creature leaves
        manager.update([]);
        expect(manager.getCreatureCount()).toBe(0);

        // Creature returns at new position
        const pos = (i + 1) * 10;
        manager.update([{ id: 1, x: pos, y: pos, rotation: 0, size: 10 }]);

        const buffer = manager.getBuffer();
        expect(manager.getCreatureCount()).toBe(1);
        // Every return should use START=END (wasn't visible last tick)
        expect(buffer[0]).toBe(pos);
        expect(buffer[2]).toBe(pos);
      }
    });

    it("should handle interleaved spawning and despawning of many creatures", () => {
      const createCreature = (id: number, pos: number) => ({
        id,
        x: pos,
        y: pos,
        rotation: 0,
        size: 10,
      });

      // Initialize with creatures 1-5
      manager.initialize([1, 2, 3, 4, 5].map((id) => createCreature(id, id * 10)));

      // Remove 2,4 and add 6,7
      manager.update([
        createCreature(1, 15),
        createCreature(3, 35),
        createCreature(5, 55),
        createCreature(6, 60),
        createCreature(7, 70),
      ]);

      expect(manager.getCreatureCount()).toBe(5);
      const buffer = manager.getBuffer();

      // Creatures 1,3,5 should interpolate normally
      expect(buffer[0]).toBe(10); // ID 1: startX = old
      expect(buffer[2]).toBe(15); // ID 1: endX = new

      // Creatures 6,7 should use START=END (new)
      expect(buffer[21]).toBe(60); // ID 6: startX = new
      expect(buffer[23]).toBe(60); // ID 6: endX = same
    });
  });
});
