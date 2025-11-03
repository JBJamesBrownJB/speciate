import { describe, it, expect } from 'vitest';
import { InterpolationCalculator } from './interpolation';
import type { InterpolatedState } from '../types/entity';

describe('InterpolationCalculator', () => {
  const calculator = new InterpolationCalculator();

  describe('calculatePosition', () => {
    it('should return previous position at alpha 0', () => {
      const state: InterpolatedState = {
        id: 'test',
        position: { x: 100, y: 100 },
        previousPosition: { x: 0, y: 0 },
        orientation: 0,
        previousOrientation: 0,
        lastUpdateTime: 1000,
        radius: 10,
      };

      const result = calculator.calculatePosition(state, 1000, 100);
      expect(result.x).toBeCloseTo(0);
      expect(result.y).toBeCloseTo(0);
    });

    it('should return current position at alpha 1', () => {
      const state: InterpolatedState = {
        id: 'test',
        position: { x: 100, y: 100 },
        previousPosition: { x: 0, y: 0 },
        orientation: 0,
        previousOrientation: 0,
        lastUpdateTime: 1000,
        radius: 10,
      };

      const result = calculator.calculatePosition(state, 1100, 100);
      expect(result.x).toBeCloseTo(100);
      expect(result.y).toBeCloseTo(100);
    });

    it('should interpolate at alpha 0.5', () => {
      const state: InterpolatedState = {
        id: 'test',
        position: { x: 100, y: 100 },
        previousPosition: { x: 0, y: 0 },
        orientation: 0,
        previousOrientation: 0,
        lastUpdateTime: 1000,
        radius: 10,
      };

      const result = calculator.calculatePosition(state, 1050, 100);
      expect(result.x).toBeCloseTo(50);
      expect(result.y).toBeCloseTo(50);
    });
  });

  describe('calculateOrientation', () => {
    it('should handle angle wrapping correctly', () => {
      const state: InterpolatedState = {
        id: 'test',
        position: { x: 0, y: 0 },
        previousPosition: { x: 0, y: 0 },
        orientation: 0.1,
        previousOrientation: Math.PI * 2 - 0.1,
        lastUpdateTime: 1000,
        radius: 10,
      };

      const result = calculator.calculateOrientation(state, 1050, 100);
      expect(Math.abs(result - (Math.PI * 2))).toBeLessThan(0.2);
    });
  });
});
