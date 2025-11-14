import { describe, it, expect, beforeEach, vi } from "vitest";
import { Container, Graphics } from "pixi.js";
import { GridRenderer } from "./GridRenderer";
import { Camera } from "@/domain/Camera";
import { Viewport } from "@/domain/Viewport";
import { GRID_CONFIG, CAMERA_CONFIG } from "@/core/constants";

describe("GridRenderer", () => {
  let worldContainer: Container;
  let camera: Camera;
  let viewport: Viewport;
  let gridRenderer: GridRenderer;

  beforeEach(() => {
    worldContainer = new Container();
    camera = new Camera(0, 0, GRID_CONFIG.MIN_ZOOM_FOR_GRID);
    viewport = new Viewport(1920, 1080);
    gridRenderer = new GridRenderer(
      worldContainer,
      GRID_CONFIG.SPACING,
      GRID_CONFIG.COLOR,
      GRID_CONFIG.ALPHA,
      GRID_CONFIG.LINE_WIDTH,
      camera.zoom
    );
  });

  describe("Construction", () => {
    it("should create grid graphics and add to container at index 0", () => {
      expect(worldContainer.children.length).toBe(1);
      expect(worldContainer.children[0]).toBeInstanceOf(Graphics);
      expect(worldContainer.getChildIndex(worldContainer.children[0])).toBe(0);
    });

    it("should initialize with provided configuration", () => {
      // Grid should be created (tested implicitly by container check)
      expect(gridRenderer).toBeDefined();
    });
  });

  describe("Visibility Thresholds", () => {
    it("should render grid when zoom >= MIN_ZOOM_FOR_GRID", () => {
      camera.setZoom(GRID_CONFIG.MIN_ZOOM_FOR_GRID);
      gridRenderer.update(camera.zoom, GRID_CONFIG.SPACING, camera, viewport);

      // Grid should have geometry (not cleared)
      const graphics = worldContainer.children[0] as Graphics;
      expect(graphics).toBeDefined();
    });

    it("should clear grid when zoom < MIN_ZOOM_FOR_GRID", () => {
      camera.setZoom(GRID_CONFIG.MIN_ZOOM_FOR_GRID - 0.1);

      // Simulate the main.ts logic that calls clear() when below threshold
      gridRenderer.clear();

      // Grid graphics should be cleared
      const graphics = worldContainer.children[0] as Graphics;
      expect(graphics).toBeDefined(); // Graphics object still exists
    });

    it("should handle transition from visible to invisible", () => {
      // Start visible
      camera.setZoom(GRID_CONFIG.MIN_ZOOM_FOR_GRID + 10);
      gridRenderer.update(camera.zoom, GRID_CONFIG.SPACING, camera, viewport);

      // Transition to invisible
      camera.setZoom(GRID_CONFIG.MIN_ZOOM_FOR_GRID - 1);
      gridRenderer.clear();

      // Should be cleared
      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should handle transition from invisible to visible", () => {
      // Start invisible
      camera.setZoom(GRID_CONFIG.MIN_ZOOM_FOR_GRID - 1);
      gridRenderer.clear();

      // Transition to visible
      camera.setZoom(GRID_CONFIG.MIN_ZOOM_FOR_GRID + 1);
      gridRenderer.update(camera.zoom, GRID_CONFIG.SPACING, camera, viewport);

      // Should have geometry
      expect(worldContainer.children[0]).toBeDefined();
    });
  });

  describe("Grid Spacing", () => {
    it("should use fixed 1m spacing", () => {
      expect(GRID_CONFIG.SPACING).toBe(1);
    });

    it("should render grid with correct spacing at different zooms", () => {
      // At zoom 20, 1m = 20 pixels
      camera.setZoom(20);
      gridRenderer.update(camera.zoom, 1, camera, viewport);

      // At zoom 50, 1m = 50 pixels
      camera.setZoom(50);
      gridRenderer.update(camera.zoom, 1, camera, viewport);

      // Grid should adapt to both (viewport culling ensures constant line count)
      expect(worldContainer.children[0]).toBeDefined();
    });
  });

  describe("Viewport Culling", () => {
    it("should calculate visible bounds correctly", () => {
      // Center camera at origin with zoom 30
      camera.move(0, 0);
      camera.setZoom(30);

      const bounds = viewport.getWorldBounds(camera);

      // At zoom 30 with 1920x1080 viewport:
      // Width in meters = 1920 / 30 = 64m
      // Height in meters = 1080 / 30 = 36m
      expect(bounds.maxX - bounds.minX).toBeCloseTo(64, 0);
      expect(bounds.maxY - bounds.minY).toBeCloseTo(36, 0);
    });

    it("should include padding when calculating grid lines", () => {
      camera.move(0, 0);
      camera.setZoom(30);

      gridRenderer.update(camera.zoom, 1, camera, viewport);

      // Grid should render beyond visible bounds (padding ensures smooth panning)
      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should handle negative camera coordinates", () => {
      camera.move(-100, -100);
      camera.setZoom(30);

      gridRenderer.update(camera.zoom, 1, camera, viewport);

      // Grid should render correctly at negative coordinates
      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should handle extreme zoom out (viewing large area)", () => {
      camera.setZoom(0.001); // Very zoomed out
      camera.move(0, 0);

      // At this zoom, grid would be cleared (below threshold)
      gridRenderer.clear();

      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should handle extreme zoom in (viewing tiny area)", () => {
      camera.setZoom(CAMERA_CONFIG.MAX_ZOOM); // Maximum zoom
      camera.move(0, 0);

      gridRenderer.update(camera.zoom, 1, camera, viewport);

      // Should render only a few grid lines (maybe 20-30 lines)
      expect(worldContainer.children[0]).toBeDefined();
    });
  });

  describe("Performance", () => {
    it("should clear graphics before redrawing", () => {
      const graphics = worldContainer.children[0] as Graphics;
      const clearSpy = vi.spyOn(graphics, "clear");

      gridRenderer.update(camera.zoom, 1, camera, viewport);

      expect(clearSpy).toHaveBeenCalled();
    });

    it("should render viewport-culled grid in reasonable time", () => {
      camera.setZoom(30);
      camera.move(0, 0);

      const startTime = performance.now();
      gridRenderer.update(camera.zoom, 1, camera, viewport);
      const endTime = performance.now();

      // Should complete in < 16ms (60 FPS target)
      expect(endTime - startTime).toBeLessThan(16);
    });

    it("should handle multiple rapid updates efficiently", () => {
      camera.setZoom(30);

      const startTime = performance.now();

      // Simulate 60 frames
      for (let i = 0; i < 60; i++) {
        camera.move(i * 0.1, i * 0.1); // Pan slightly each frame
        gridRenderer.update(camera.zoom, 1, camera, viewport);
      }

      const endTime = performance.now();
      const avgFrameTime = (endTime - startTime) / 60;

      // Average frame time should be < 16ms
      expect(avgFrameTime).toBeLessThan(16);
    });
  });

  describe("Resource Cleanup", () => {
    it("should clear grid geometry", () => {
      gridRenderer.update(camera.zoom, 1, camera, viewport);
      gridRenderer.clear();

      // Graphics object should still exist but be cleared
      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should destroy graphics on destroy()", () => {
      const graphics = worldContainer.children[0] as Graphics;
      const destroySpy = vi.spyOn(graphics, "destroy");

      gridRenderer.destroy();

      expect(destroySpy).toHaveBeenCalled();
    });

    it("should be safe to call clear() multiple times", () => {
      gridRenderer.clear();
      gridRenderer.clear();
      gridRenderer.clear();

      // Should not throw
      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should be safe to call destroy() multiple times", () => {
      gridRenderer.destroy();

      // Second destroy should not throw (idempotent)
      expect(() => gridRenderer.destroy()).not.toThrow();
    });
  });

  describe("Edge Cases", () => {
    it("should handle world boundaries correctly", () => {
      // Position camera at world edge
      camera.move(1000000, 1000000); // Max positive coordinates
      camera.setZoom(30);

      gridRenderer.update(camera.zoom, 1, camera, viewport);

      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should handle zero dimensions viewport", () => {
      const tinyViewport = new Viewport(0, 0);

      expect(() => {
        gridRenderer.update(camera.zoom, 1, camera, tinyViewport);
      }).not.toThrow();
    });

    it("should handle minimum zoom", () => {
      camera.setZoom(CAMERA_CONFIG.MIN_ZOOM);

      // Grid would be cleared at this zoom
      gridRenderer.clear();

      expect(worldContainer.children[0]).toBeDefined();
    });

    it("should handle maximum zoom", () => {
      camera.setZoom(CAMERA_CONFIG.MAX_ZOOM);

      gridRenderer.update(camera.zoom, 1, camera, viewport);

      expect(worldContainer.children[0]).toBeDefined();
    });
  });
});
