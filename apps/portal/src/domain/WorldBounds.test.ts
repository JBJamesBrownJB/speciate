import { describe, it, expect } from "vitest";
import {
  createWorldBounds,
  worldBoundsFromDimensions,
  worldBoundsWidth,
  worldBoundsHeight,
  worldBoundsContains,
} from "./WorldBounds";

describe("WorldBounds", () => {
  describe("createWorldBounds", () => {
    it("should create bounds with explicit min/max values", () => {
      const bounds = createWorldBounds(-100, 100, -50, 50);
      expect(bounds.minX).toBe(-100);
      expect(bounds.maxX).toBe(100);
      expect(bounds.minY).toBe(-50);
      expect(bounds.maxY).toBe(50);
    });

    it("should create bounds with all positive values", () => {
      const bounds = createWorldBounds(0, 1000, 0, 500);
      expect(bounds.minX).toBe(0);
      expect(bounds.maxX).toBe(1000);
      expect(bounds.minY).toBe(0);
      expect(bounds.maxY).toBe(500);
    });

    it("should create bounds with all negative values", () => {
      const bounds = createWorldBounds(-1000, -100, -500, -50);
      expect(bounds.minX).toBe(-1000);
      expect(bounds.maxX).toBe(-100);
      expect(bounds.minY).toBe(-500);
      expect(bounds.maxY).toBe(-50);
    });
  });

  describe("worldBoundsFromDimensions", () => {
    it("should create centered bounds from width and height", () => {
      const bounds = worldBoundsFromDimensions(200, 100);
      expect(bounds.minX).toBe(-100);
      expect(bounds.maxX).toBe(100);
      expect(bounds.minY).toBe(-50);
      expect(bounds.maxY).toBe(50);
    });

    it("should handle square dimensions", () => {
      const bounds = worldBoundsFromDimensions(1000, 1000);
      expect(bounds.minX).toBe(-500);
      expect(bounds.maxX).toBe(500);
      expect(bounds.minY).toBe(-500);
      expect(bounds.maxY).toBe(500);
    });

    it("should create bounds centered at origin", () => {
      const bounds = worldBoundsFromDimensions(10000, 10000);
      expect(bounds.minX).toBe(-5000);
      expect(bounds.maxX).toBe(5000);
      expect(bounds.minY).toBe(-5000);
      expect(bounds.maxY).toBe(5000);
    });
  });

  describe("worldBoundsWidth", () => {
    it("should calculate width from centered bounds", () => {
      const bounds = createWorldBounds(-100, 100, -50, 50);
      expect(worldBoundsWidth(bounds)).toBe(200);
    });

    it("should calculate width from positive-only bounds", () => {
      const bounds = createWorldBounds(0, 1000, 0, 500);
      expect(worldBoundsWidth(bounds)).toBe(1000);
    });
  });

  describe("worldBoundsHeight", () => {
    it("should calculate height from centered bounds", () => {
      const bounds = createWorldBounds(-100, 100, -50, 50);
      expect(worldBoundsHeight(bounds)).toBe(100);
    });

    it("should calculate height from positive-only bounds", () => {
      const bounds = createWorldBounds(0, 1000, 0, 500);
      expect(worldBoundsHeight(bounds)).toBe(500);
    });
  });

  describe("worldBoundsContains", () => {
    const bounds = createWorldBounds(-100, 100, -50, 50);

    it("should return true for point at origin", () => {
      expect(worldBoundsContains(bounds, 0, 0)).toBe(true);
    });

    it("should return true for point at edges", () => {
      expect(worldBoundsContains(bounds, 100, 50)).toBe(true);
      expect(worldBoundsContains(bounds, -100, -50)).toBe(true);
      expect(worldBoundsContains(bounds, -100, 50)).toBe(true);
      expect(worldBoundsContains(bounds, 100, -50)).toBe(true);
    });

    it("should return true for point inside bounds", () => {
      expect(worldBoundsContains(bounds, 25, -25)).toBe(true);
      expect(worldBoundsContains(bounds, -50, 25)).toBe(true);
    });

    it("should return false for point outside X bounds", () => {
      expect(worldBoundsContains(bounds, 101, 0)).toBe(false);
      expect(worldBoundsContains(bounds, -101, 0)).toBe(false);
    });

    it("should return false for point outside Y bounds", () => {
      expect(worldBoundsContains(bounds, 0, 51)).toBe(false);
      expect(worldBoundsContains(bounds, 0, -51)).toBe(false);
    });
  });
});
