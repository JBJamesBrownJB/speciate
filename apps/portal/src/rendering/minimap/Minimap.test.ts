import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { Container } from "pixi.js";
import { Minimap } from "./Minimap";
import { createWorldBounds } from "@/domain/WorldBounds";

describe("Minimap", () => {
  let minimap: Minimap;
  let parentContainer: Container;
  const worldBounds = createWorldBounds(-5000, 5000, -5000, 5000);

  beforeEach(() => {
    parentContainer = new Container();
    minimap = new Minimap(parentContainer, worldBounds, 180);
  });

  afterEach(() => {
    minimap.destroy();
  });

  describe("initialization", () => {
    it("should be visible by default", () => {
      expect(minimap.isVisible()).toBe(true);
    });

    it("should have correct config", () => {
      expect(minimap.config.name).toBe("minimap");
      expect(minimap.config.devToolsOnly).toBe(false);
      expect(minimap.config.keyboardShortcut).toBe("m");
    });

    it("should add container to parent", () => {
      expect(parentContainer.children.length).toBeGreaterThan(0);
    });

    it("should throw for zero size", () => {
      expect(() => new Minimap(new Container(), worldBounds, 0)).toThrow(
        "Minimap size must be positive"
      );
    });

    it("should throw for negative size", () => {
      expect(() => new Minimap(new Container(), worldBounds, -100)).toThrow(
        "Minimap size must be positive"
      );
    });

    it("should throw for invalid worldBounds (minX >= maxX)", () => {
      const invalidBounds = createWorldBounds(5000, -5000, -5000, 5000);
      expect(() => new Minimap(new Container(), invalidBounds, 180)).toThrow(
        "WorldBounds must have positive dimensions"
      );
    });

    it("should throw for invalid worldBounds (minY >= maxY)", () => {
      const invalidBounds = createWorldBounds(-5000, 5000, 5000, -5000);
      expect(() => new Minimap(new Container(), invalidBounds, 180)).toThrow(
        "WorldBounds must have positive dimensions"
      );
    });
  });

  describe("show/hide", () => {
    it("should hide when hide() called", () => {
      minimap.hide();
      expect(minimap.isVisible()).toBe(false);
    });

    it("should show when show() called after hiding", () => {
      minimap.hide();
      minimap.show();
      expect(minimap.isVisible()).toBe(true);
    });

    it("should toggle visibility", () => {
      expect(minimap.isVisible()).toBe(true);
      minimap.toggle();
      expect(minimap.isVisible()).toBe(false);
      minimap.toggle();
      expect(minimap.isVisible()).toBe(true);
    });
  });

  describe("update", () => {
    it("should not throw when updating", () => {
      const mockCamera = { x: 0, y: 0, zoom: 10 };
      expect(() => minimap.update(mockCamera, 800, 600)).not.toThrow();
    });

    it("should handle zero zoom gracefully", () => {
      const zeroZoom = { x: 0, y: 0, zoom: 0 };
      minimap.update({ x: 0, y: 0, zoom: 10 }, 800, 600);
      const rectBefore = minimap.getViewportRect();

      expect(() => minimap.update(zeroZoom, 800, 600)).not.toThrow();

      const rectAfter = minimap.getViewportRect();
      expect(rectAfter).toEqual(rectBefore);
    });

    it("should handle negative zoom gracefully", () => {
      const negativeZoom = { x: 0, y: 0, zoom: -5 };
      minimap.update({ x: 0, y: 0, zoom: 10 }, 800, 600);
      const rectBefore = minimap.getViewportRect();

      expect(() => minimap.update(negativeZoom, 800, 600)).not.toThrow();

      const rectAfter = minimap.getViewportRect();
      expect(rectAfter).toEqual(rectBefore);
    });

    it("should be idempotent after destroy", () => {
      minimap.destroy();
      expect(() => minimap.update({ x: 0, y: 0, zoom: 10 }, 800, 600)).not.toThrow();
    });
  });

  describe("click interaction", () => {
    it("should have onMinimapClick callback support", () => {
      expect(minimap.onMinimapClick).toBeNull();
    });

    it("should call onMinimapClick with world coordinates when clicked", () => {
      const callback = vi.fn();
      minimap.onMinimapClick = callback;

      minimap.handleClick(90, 90);

      expect(callback).toHaveBeenCalledWith(0, 0);
    });

    it("should convert minimap coordinates to world coordinates correctly", () => {
      const callback = vi.fn();
      minimap.onMinimapClick = callback;

      minimap.handleClick(0, 0);
      expect(callback).toHaveBeenCalledWith(-5000, -5000);

      minimap.handleClick(180, 180);
      expect(callback).toHaveBeenCalledWith(5000, 5000);
    });
  });

  describe("viewport rectangle", () => {
    it("should calculate viewport rectangle based on camera position and zoom", () => {
      const mockCamera = { x: 0, y: 0, zoom: 10 };

      minimap.update(mockCamera, 800, 600);

      const viewportRect = minimap.getViewportRect();

      expect(viewportRect.x).toBeCloseTo(90 - (800 / 10 / 10000) * 180 / 2, 1);
      expect(viewportRect.y).toBeCloseTo(90 - (600 / 10 / 10000) * 180 / 2, 1);
    });

    it("should show larger viewport rectangle when zoomed out", () => {
      const zoomedOut = { x: 0, y: 0, zoom: 5 };
      const zoomedIn = { x: 0, y: 0, zoom: 20 };

      minimap.update(zoomedOut, 800, 600);
      const rectZoomedOut = minimap.getViewportRect();

      minimap.update(zoomedIn, 800, 600);
      const rectZoomedIn = minimap.getViewportRect();

      expect(rectZoomedOut.width).toBeGreaterThan(rectZoomedIn.width);
      expect(rectZoomedOut.height).toBeGreaterThan(rectZoomedIn.height);
    });

    it("should move viewport rectangle when camera pans", () => {
      minimap.update({ x: -2500, y: -2500, zoom: 10 }, 800, 600);
      const rectTopLeft = minimap.getViewportRect();

      minimap.update({ x: 2500, y: 2500, zoom: 10 }, 800, 600);
      const rectBottomRight = minimap.getViewportRect();

      expect(rectBottomRight.x).toBeGreaterThan(rectTopLeft.x);
      expect(rectBottomRight.y).toBeGreaterThan(rectTopLeft.y);
    });
  });

  describe("destroy", () => {
    it("should not throw when destroyed", () => {
      expect(() => minimap.destroy()).not.toThrow();
    });

    it("should not throw when destroyed twice", () => {
      minimap.destroy();
      expect(() => minimap.destroy()).not.toThrow();
    });
  });

  describe("setPosition", () => {
    it("should set position of minimap container", () => {
      minimap.setPosition(100, 200);

      const container = (minimap as any).minimapContainer;
      expect(container.x).toBe(100);
      expect(container.y).toBe(200);
    });
  });
});
