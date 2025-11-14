import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Camera } from './Camera';
import { CAMERA_CONFIG } from '../core/constants';

describe('Camera', () => {
  let camera: Camera;

  beforeEach(() => {
    // Create camera at origin with 10px per meter zoom
    camera = new Camera(0, 0, 10);
  });

  describe('construction', () => {
    it('should initialize with given position and zoom', () => {
      expect(camera.x).toBe(0);
      expect(camera.y).toBe(0);
      expect(camera.zoom).toBe(10); // 10 pixels per meter
    });

    it('should clamp zoom to minimum', () => {
      const cam = new Camera(0, 0, CAMERA_CONFIG.MIN_ZOOM / 2);
      expect(cam.zoom).toBe(CAMERA_CONFIG.MIN_ZOOM);
    });

    it('should clamp zoom to maximum', () => {
      const cam = new Camera(0, 0, CAMERA_CONFIG.MAX_ZOOM * 2);
      expect(cam.zoom).toBe(CAMERA_CONFIG.MAX_ZOOM);
    });
  });

  describe('move', () => {
    it('should update camera position', () => {
      camera.move(10, 20);
      expect(camera.x).toBe(10);
      expect(camera.y).toBe(20);
    });

    it('should allow negative positions', () => {
      camera.move(-100, -200);
      expect(camera.x).toBe(-100);
      expect(camera.y).toBe(-200);
    });

    it('should allow large positions (within world limit)', () => {
      camera.move(500000, -500000);
      expect(camera.x).toBe(500000);
      expect(camera.y).toBe(-500000);
    });
  });

  describe('setZoom', () => {
    it('should update zoom level', () => {
      camera.setZoom(50);
      expect(camera.zoom).toBe(50);
    });

    it('should clamp zoom to minimum', () => {
      camera.setZoom(CAMERA_CONFIG.MIN_ZOOM / 2);
      expect(camera.zoom).toBe(CAMERA_CONFIG.MIN_ZOOM);
    });

    it('should clamp zoom to maximum', () => {
      camera.setZoom(CAMERA_CONFIG.MAX_ZOOM * 2);
      expect(camera.zoom).toBe(CAMERA_CONFIG.MAX_ZOOM);
    });

    it('should handle exact boundary values', () => {
      camera.setZoom(CAMERA_CONFIG.MIN_ZOOM);
      expect(camera.zoom).toBe(CAMERA_CONFIG.MIN_ZOOM);

      camera.setZoom(CAMERA_CONFIG.MAX_ZOOM);
      expect(camera.zoom).toBe(CAMERA_CONFIG.MAX_ZOOM);
    });
  });

  describe('worldToScreen', () => {
    it('should convert world coordinates to screen coordinates', () => {
      // Camera at (0, 0), zoom 10px/m
      // World point (5, 3) should be (50, 30) on screen
      const screen = camera.worldToScreen(5, 3);
      expect(screen.x).toBe(50);
      expect(screen.y).toBe(30);
    });

    it('should account for camera position', () => {
      camera.move(10, 5);
      // World point (15, 8) is 5m right, 3m down from camera
      // At 10px/m zoom, that's (50, 30) on screen
      const screen = camera.worldToScreen(15, 8);
      expect(screen.x).toBe(50);
      expect(screen.y).toBe(30);
    });

    it('should account for zoom level', () => {
      camera.setZoom(20); // 20 pixels per meter
      // World point (5, 3) should be (100, 60) on screen
      const screen = camera.worldToScreen(5, 3);
      expect(screen.x).toBe(100);
      expect(screen.y).toBe(60);
    });

    it('should handle negative world coordinates', () => {
      camera.move(10, 10);
      camera.setZoom(10);
      // World point (5, 5) is 5m left, 5m up from camera (10, 10)
      // That's (-5, -5) relative, which is (-50, -50) on screen
      const screen = camera.worldToScreen(5, 5);
      expect(screen.x).toBe(-50);
      expect(screen.y).toBe(-50);
    });
  });

  describe('screenToWorld', () => {
    it('should convert screen coordinates to world coordinates', () => {
      // Camera at (0, 0), zoom 10px/m
      // Screen point (50, 30) should be (5, 3) in world
      const world = camera.screenToWorld(50, 30);
      expect(world.x).toBe(5);
      expect(world.y).toBe(3);
    });

    it('should account for camera position', () => {
      camera.move(10, 5);
      camera.setZoom(10);
      // Screen point (50, 30) is 5m right, 3m down from camera
      // Camera is at (10, 5), so world point is (15, 8)
      const world = camera.screenToWorld(50, 30);
      expect(world.x).toBe(15);
      expect(world.y).toBe(8);
    });

    it('should account for zoom level', () => {
      camera.setZoom(20); // 20 pixels per meter
      // Screen point (100, 60) should be (5, 3) in world
      const world = camera.screenToWorld(100, 60);
      expect(world.x).toBe(5);
      expect(world.y).toBe(3);
    });

    it('should be inverse of worldToScreen', () => {
      camera.move(100, -50);
      camera.setZoom(25);

      const worldPoint = { x: 150, y: 75 };
      const screen = camera.worldToScreen(worldPoint.x, worldPoint.y);
      const backToWorld = camera.screenToWorld(screen.x, screen.y);

      expect(backToWorld.x).toBeCloseTo(worldPoint.x, 5);
      expect(backToWorld.y).toBeCloseTo(worldPoint.y, 5);
    });
  });

  describe('deltaMove', () => {
    it('should move camera by relative amount', () => {
      camera.move(10, 20);
      camera.deltaMove(5, 3);
      expect(camera.x).toBe(15);
      expect(camera.y).toBe(23);
    });

    it('should handle negative deltas', () => {
      camera.move(10, 20);
      camera.deltaMove(-5, -10);
      expect(camera.x).toBe(5);
      expect(camera.y).toBe(10);
    });
  });

  describe('adjustZoom', () => {
    it('should adjust zoom by a factor', () => {
      camera.setZoom(10);
      camera.adjustZoom(2); // Double the zoom
      expect(camera.zoom).toBe(20);
    });

    it('should handle zoom out (factor < 1)', () => {
      camera.setZoom(50);
      camera.adjustZoom(0.5); // Half the zoom
      expect(camera.zoom).toBe(25);
    });

    it('should respect zoom limits when adjusting', () => {
      camera.setZoom(250);
      camera.adjustZoom(2); // Would be 500, but clamped to MAX_ZOOM (400)
      expect(camera.zoom).toBe(CAMERA_CONFIG.MAX_ZOOM);
    });
  });

  describe('applyTransform', () => {
    it('should apply zoom as uniform scale to container', () => {
      const container = {
        scale: { set: vi.fn() },
        position: { set: vi.fn() }
      };

      camera.setZoom(20);
      camera.applyTransform(container, 800, 600);

      // Should set uniform scale to zoom level
      expect(container.scale.set).toHaveBeenCalledWith(20);
    });

    it('should center camera at (0,0) in middle of screen', () => {
      const container = {
        scale: { set: vi.fn() },
        position: { set: vi.fn() }
      };

      camera.move(0, 0);
      camera.setZoom(10);
      camera.applyTransform(container, 800, 600);

      // When camera is at origin, container should be centered
      expect(container.position.set).toHaveBeenCalledWith(400, 300);
    });

    it('should offset container position based on camera position', () => {
      const container = {
        scale: { set: vi.fn() },
        position: { set: vi.fn() }
      };

      camera.move(10, 5); // Camera moves 10m right, 5m down
      camera.setZoom(10);
      camera.applyTransform(container, 800, 600);

      // Container should shift left/up to keep camera centered
      // Center (400, 300) - camera offset (100px, 50px) = (300, 250)
      expect(container.position.set).toHaveBeenCalledWith(300, 250);
    });

    it('should account for zoom in position calculation', () => {
      const container = {
        scale: { set: vi.fn() },
        position: { set: vi.fn() }
      };

      camera.move(10, 5);
      camera.setZoom(20); // Higher zoom = larger pixel offset
      camera.applyTransform(container, 800, 600);

      // Center (400, 300) - camera offset (200px, 100px) = (200, 200)
      expect(container.position.set).toHaveBeenCalledWith(200, 200);
    });

    it('should handle negative camera positions', () => {
      const container = {
        scale: { set: vi.fn() },
        position: { set: vi.fn() }
      };

      camera.move(-10, -5); // Camera moves left/up
      camera.setZoom(10);
      camera.applyTransform(container, 800, 600);

      // Container should shift right/down
      // Center (400, 300) - camera offset (-100px, -50px) = (500, 350)
      expect(container.position.set).toHaveBeenCalledWith(500, 350);
    });

    it('should work with different screen sizes', () => {
      const container = {
        scale: { set: vi.fn() },
        position: { set: vi.fn() }
      };

      camera.move(0, 0);
      camera.setZoom(10);
      camera.applyTransform(container, 1920, 1080);

      // Should center in larger screen
      expect(container.position.set).toHaveBeenCalledWith(960, 540);
    });
  });
});
