import { describe, it, expect } from 'vitest';
import { SpatialQuery } from './SpatialQuery';

describe('SpatialQuery', () => {
  describe('distance', () => {
    it('should calculate distance between two points', () => {
      const result = SpatialQuery.distance(0, 0, 3, 4);
      expect(result).toBe(5);
    });

    it('should calculate distance when points are the same', () => {
      const result = SpatialQuery.distance(5, 10, 5, 10);
      expect(result).toBe(0);
    });

    it('should calculate distance with negative coordinates', () => {
      const result = SpatialQuery.distance(-3, -4, 0, 0);
      expect(result).toBe(5);
    });

    it('should calculate distance in any direction', () => {
      const result = SpatialQuery.distance(10, 10, 7, 6);
      expect(result).toBe(5);
    });
  });

  describe('isInViewport', () => {
    const viewportBounds = {
      minX: 0,
      maxX: 100,
      minY: 0,
      maxY: 100
    };

    it('should return true when entity is fully inside viewport', () => {
      const entity = { x: 50, y: 50, width: 10, height: 10 };
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });

    it('should return true when entity is at viewport center', () => {
      const entity = { x: 50, y: 50, width: 20, height: 20 };
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });

    it('should return true when entity partially overlaps left edge', () => {
      const entity = { x: 5, y: 50, width: 12, height: 10 }; // Left edge at -1
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });

    it('should return true when entity partially overlaps right edge', () => {
      const entity = { x: 95, y: 50, width: 12, height: 10 }; // Right edge at 101
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });

    it('should return true when entity partially overlaps top edge', () => {
      const entity = { x: 50, y: 5, width: 10, height: 12 }; // Top edge at -1
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });

    it('should return true when entity partially overlaps bottom edge', () => {
      const entity = { x: 50, y: 95, width: 10, height: 12 }; // Bottom edge at 101
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });

    it('should return false when entity is completely to the left', () => {
      const entity = { x: -20, y: 50, width: 10, height: 10 }; // Right edge at -15
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(false);
    });

    it('should return false when entity is completely to the right', () => {
      const entity = { x: 120, y: 50, width: 10, height: 10 }; // Left edge at 115
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(false);
    });

    it('should return false when entity is completely above', () => {
      const entity = { x: 50, y: -20, width: 10, height: 10 }; // Bottom edge at -15
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(false);
    });

    it('should return false when entity is completely below', () => {
      const entity = { x: 50, y: 120, width: 10, height: 10 }; // Top edge at 115
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(false);
    });

    it('should return true when entity touches viewport edge exactly', () => {
      const entity = { x: 5, y: 5, width: 10, height: 10 }; // Edges at 0, 10
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });

    it('should handle large entities that encompass entire viewport', () => {
      const entity = { x: 50, y: 50, width: 200, height: 200 };
      expect(SpatialQuery.isInViewport(entity, viewportBounds)).toBe(true);
    });
  });
});
