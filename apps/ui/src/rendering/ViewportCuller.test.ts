import { describe, it, expect } from 'vitest';
import { ViewportCuller } from './ViewportCuller';

describe('ViewportCuller', () => {
  const culler = new ViewportCuller(100);

  describe('calculateBounds', () => {
    it('should calculate viewport bounds with padding', () => {
      const bounds = culler.calculateBounds(800, 600, 0, 0);

      expect(bounds.minX).toBe(-100);
      expect(bounds.maxX).toBe(900);
      expect(bounds.minY).toBe(-100);
      expect(bounds.maxY).toBe(700);
    });
  });

  describe('isVisible', () => {
    const bounds = {
      minX: 0,
      maxX: 800,
      minY: 0,
      maxY: 600,
    };

    it('should return true for entity inside viewport', () => {
      const position = { x: 400, y: 300 };
      expect(culler.isVisible(position, 10, bounds)).toBe(true);
    });

    it('should return false for entity outside viewport', () => {
      const position = { x: 1000, y: 300 };
      expect(culler.isVisible(position, 10, bounds)).toBe(false);
    });

    it('should return true for entity partially visible', () => {
      const position = { x: 795, y: 300 };
      expect(culler.isVisible(position, 10, bounds)).toBe(true);
    });

    it('should account for radius in visibility check', () => {
      const position = { x: 810, y: 300 };
      expect(culler.isVisible(position, 20, bounds)).toBe(true);
    });
  });
});
