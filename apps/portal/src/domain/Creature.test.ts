import { describe, it, expect } from 'vitest';
import { Creature } from './Creature';

describe('Creature', () => {
  describe('construction', () => {
    it('should create a creature with all required properties', () => {
      const creature = new Creature(1, 100, 200, 1.5, 2.0, 1.5);

      expect(creature.id).toBe(1);
      expect(creature.x).toBe(100);
      expect(creature.y).toBe(200);
      expect(creature.rotation).toBe(1.5);
      expect(creature.width).toBe(2.0);
      expect(creature.height).toBe(1.5);
    });

    it('should handle zero position', () => {
      const creature = new Creature(1, 0, 0, 0, 1, 1);

      expect(creature.x).toBe(0);
      expect(creature.y).toBe(0);
      expect(creature.rotation).toBe(0);
    });

    it('should handle negative positions', () => {
      const creature = new Creature(2, -500, -300, -Math.PI, 1, 1);

      expect(creature.x).toBe(-500);
      expect(creature.y).toBe(-300);
      expect(creature.rotation).toBe(-Math.PI);
    });

    it('should handle large world coordinates', () => {
      const creature = new Creature(3, 999999, -999999, 0, 1, 1);

      expect(creature.x).toBe(999999);
      expect(creature.y).toBe(-999999);
    });
  });

  describe('withTransform', () => {
    it('should create a new creature with updated position and rotation', () => {
      const original = new Creature(1, 100, 200, 0, 1, 1);
      const updated = original.withTransform(150, 250, Math.PI);

      expect(updated.x).toBe(150);
      expect(updated.y).toBe(250);
      expect(updated.rotation).toBe(Math.PI);

      // Original should be unchanged
      expect(original.x).toBe(100);
      expect(original.y).toBe(200);
      expect(original.rotation).toBe(0);
    });
  });

  describe('immutability', () => {
    it('should not allow direct mutation of properties', () => {
      const creature = new Creature(1, 100, 200, 0, 1, 1);

      // TypeScript will prevent this at compile time, but let's verify at runtime
      expect(() => {
        (creature as any).x = 500;
      }).toThrow();
    });
  });

  describe('fromMessage', () => {
    it('should create creature from network message format', () => {
      const message = {
        id: 42,
        x: 123.456,
        y: -789.012,
        rotation: 1.23,
        width: 2.5,
        height: 1.8
      };

      const creature = Creature.fromMessage(message);

      expect(creature.id).toBe(42);
      expect(creature.x).toBe(123.456);
      expect(creature.y).toBe(-789.012);
      expect(creature.rotation).toBe(1.23);
      expect(creature.width).toBe(2.5);
      expect(creature.height).toBe(1.8);
    });

    it('should handle optional properties with defaults', () => {
      const message = {
        id: 1,
        x: 0,
        y: 0
      };

      const creature = Creature.fromMessage(message);

      expect(creature.rotation).toBe(0);
      expect(creature.width).toBe(1);
      expect(creature.height).toBe(1);
    });
  });
});
