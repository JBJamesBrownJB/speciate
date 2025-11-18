import { describe, it, expect } from 'vitest';
import { Viewport } from './Viewport';
import { Camera } from './Camera';
import { Creature } from './Creature';

describe('Viewport', () => {
  describe('construction', () => {
    it('should create viewport with given dimensions', () => {
      const viewport = new Viewport(800, 600);

      expect(viewport.width).toBe(800);
      expect(viewport.height).toBe(600);
    });
  });

  describe('getWorldBounds', () => {
    it('should calculate world bounds from camera', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10); // 10 pixels per meter

      const bounds = viewport.getWorldBounds(camera);

      // At 10px/m, 800px = 80m width, 600px = 60m height
      // Centered at (0, 0), so:
      // minX = -40, maxX = 40, minY = -30, maxY = 30
      expect(bounds.minX).toBe(-40);
      expect(bounds.maxX).toBe(40);
      expect(bounds.minY).toBe(-30);
      expect(bounds.maxY).toBe(30);
    });

    it('should account for camera position', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(100, 50, 10);

      const bounds = viewport.getWorldBounds(camera);

      // Camera at (100, 50), viewport shows ±40m horizontally, ±30m vertically
      expect(bounds.minX).toBe(60); // 100 - 40
      expect(bounds.maxX).toBe(140); // 100 + 40
      expect(bounds.minY).toBe(20); // 50 - 30
      expect(bounds.maxY).toBe(80); // 50 + 30
    });

    it('should account for zoom level', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 20); // 20 pixels per meter (zoomed in)

      const bounds = viewport.getWorldBounds(camera);

      // At 20px/m, 800px = 40m width, 600px = 30m height
      expect(bounds.minX).toBe(-20);
      expect(bounds.maxX).toBe(20);
      expect(bounds.minY).toBe(-15);
      expect(bounds.maxY).toBe(15);
    });
  });

  describe('isCreatureVisible', () => {
    it('should return true for creature inside viewport', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      // Creature at origin should be visible
      const creature = new Creature(1, 0, 0, 0, 1);

      expect(viewport.isCreatureVisible(creature, camera)).toBe(true);
    });

    it('should return true for creature partially in viewport', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      // Creature at edge of viewport (world bounds: -40 to 40, -30 to 30)
      const creature = new Creature(1, 35, 0, 0, 10); // 10m size, extends to 40

      expect(viewport.isCreatureVisible(creature, camera)).toBe(true);
    });

    it('should return false for creature completely outside viewport', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      // Creature far outside viewport (world bounds: -40 to 40, -30 to 30)
      const creature = new Creature(1, 100, 100, 0, 1);

      expect(viewport.isCreatureVisible(creature, camera)).toBe(false);
    });

    it('should account for creature size in visibility check', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      // Creature center is outside, but its size brings it partially in
      // World bounds: -40 to 40, -30 to 30
      // Creature at (45, 0) with width 12m extends from 39 to 51
      const creature = new Creature(1, 45, 0, 0, 12);

      expect(viewport.isCreatureVisible(creature, camera)).toBe(true);
    });

    it('should handle creatures with zero size', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      const creature = new Creature(1, 0, 0, 0, 0);

      expect(viewport.isCreatureVisible(creature, camera)).toBe(true);
    });

    it('should work with moved camera', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(1000, 500, 10);

      // World bounds: 960 to 1040, 470 to 530
      const visibleCreature = new Creature(1, 1000, 500, 0, 1);
      const invisibleCreature = new Creature(2, 0, 0, 0, 1);

      expect(viewport.isCreatureVisible(visibleCreature, camera)).toBe(true);
      expect(viewport.isCreatureVisible(invisibleCreature, camera)).toBe(false);
    });
  });

  describe('cullCreatures', () => {
    it('should return only visible creatures', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      const creatures = [
        new Creature(1, 0, 0, 0, 1),     // Visible (center)
        new Creature(2, 35, 25, 0, 1),   // Visible (near edge)
        new Creature(3, 100, 100, 0, 1), // Invisible (far away)
        new Creature(4, -100, 0, 0, 1)   // Invisible (far away)
      ];

      const visible = viewport.cullCreatures(creatures, camera);

      expect(visible).toHaveLength(2);
      expect(visible.map(c => c.id)).toEqual([1, 2]);
    });

    it('should return empty array when no creatures visible', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      const creatures = [
        new Creature(1, 1000, 1000, 0, 1),
        new Creature(2, -1000, -1000, 0, 1)
      ];

      const visible = viewport.cullCreatures(creatures, camera);

      expect(visible).toHaveLength(0);
    });

    it('should return all creatures when all visible', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      const creatures = [
        new Creature(1, 0, 0, 0, 1),
        new Creature(2, 10, 10, 0, 1),
        new Creature(3, -10, -10, 0, 1)
      ];

      const visible = viewport.cullCreatures(creatures, camera);

      expect(visible).toHaveLength(3);
    });

    it('should handle empty input', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      const visible = viewport.cullCreatures([], camera);

      expect(visible).toHaveLength(0);
    });
  });

  describe('resize', () => {
    it('should update viewport dimensions', () => {
      const viewport = new Viewport(800, 600);

      viewport.resize(1024, 768);

      expect(viewport.width).toBe(1024);
      expect(viewport.height).toBe(768);
    });

    it('should affect world bounds calculation', () => {
      const viewport = new Viewport(800, 600);
      const camera = new Camera(0, 0, 10);

      const boundsBefore = viewport.getWorldBounds(camera);

      viewport.resize(1600, 1200); // Double the size

      const boundsAfter = viewport.getWorldBounds(camera);

      // World bounds should be doubled
      expect(boundsAfter.maxX - boundsAfter.minX).toBe(
        (boundsBefore.maxX - boundsBefore.minX) * 2
      );
      expect(boundsAfter.maxY - boundsAfter.minY).toBe(
        (boundsBefore.maxY - boundsBefore.minY) * 2
      );
    });
  });
});
