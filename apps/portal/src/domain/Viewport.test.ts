import { describe, it, expect } from 'vitest';
import { Viewport } from './Viewport';
import { Camera } from './Camera';

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
