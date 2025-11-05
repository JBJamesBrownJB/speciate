import { describe, it, expect } from 'vitest';
import {
  isSimulationStateMessage,
  isSimulationFrame,
  adaptSimulationFrame,
  type SimulationFrame,
  type AgentTransform,
  type SimulationStateMessage,
} from './messages';

describe('messages', () => {
  describe('isSimulationStateMessage', () => {
    it('should return true for valid SimulationStateMessage', () => {
      const valid: SimulationStateMessage = {
        tick: 100,
        server_time: 1234567890,
        creatures: [],
      };
      expect(isSimulationStateMessage(valid)).toBe(true);
    });

    it('should return false for invalid SimulationStateMessage (missing tick)', () => {
      const invalid = {
        server_time: 1234567890,
        creatures: [],
      };
      expect(isSimulationStateMessage(invalid)).toBe(false);
    });

    it('should return false for invalid SimulationStateMessage (missing creatures)', () => {
      const invalid = {
        tick: 100,
        server_time: 1234567890,
      };
      expect(isSimulationStateMessage(invalid)).toBe(false);
    });

    it('should return false for null', () => {
      expect(isSimulationStateMessage(null)).toBe(false);
    });

    it('should return false for undefined', () => {
      expect(isSimulationStateMessage(undefined)).toBe(false);
    });

    it('should return false for non-object types', () => {
      expect(isSimulationStateMessage('string')).toBe(false);
      expect(isSimulationStateMessage(123)).toBe(false);
      expect(isSimulationStateMessage(true)).toBe(false);
    });
  });

  describe('isSimulationFrame', () => {
    it('should return true for valid SimulationFrame', () => {
      const valid: SimulationFrame = {
        tick: 100,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      };
      expect(isSimulationFrame(valid)).toBe(true);
    });

    it('should return true for SimulationFrame with agents', () => {
      const valid: SimulationFrame = {
        tick: 100,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 1, x: 10, y: 20, vx: 1, vy: 2, rotation: 0.5 },
          { id: 2, x: 30, y: 40, vx: -1, vy: 1, rotation: 1.5 },
        ],
      };
      expect(isSimulationFrame(valid)).toBe(true);
    });

    it('should return false for invalid SimulationFrame (missing tick)', () => {
      const invalid = {
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      };
      expect(isSimulationFrame(invalid)).toBe(false);
    });

    it('should return false for invalid SimulationFrame (missing timestamp)', () => {
      const invalid = {
        tick: 100,
        agents: [],
      };
      expect(isSimulationFrame(invalid)).toBe(false);
    });

    it('should return false for invalid SimulationFrame (missing agents)', () => {
      const invalid = {
        tick: 100,
        timestamp: '2025-01-05T21:00:00.000Z',
      };
      expect(isSimulationFrame(invalid)).toBe(false);
    });

    it('should return false for invalid SimulationFrame (agents not array)', () => {
      const invalid = {
        tick: 100,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: 'not-an-array',
      };
      expect(isSimulationFrame(invalid)).toBe(false);
    });

    it('should return false for invalid SimulationFrame (timestamp not string)', () => {
      const invalid = {
        tick: 100,
        timestamp: 1234567890,
        agents: [],
      };
      expect(isSimulationFrame(invalid)).toBe(false);
    });

    it('should return false for null', () => {
      expect(isSimulationFrame(null)).toBe(false);
    });

    it('should return false for undefined', () => {
      expect(isSimulationFrame(undefined)).toBe(false);
    });

    it('should return false for non-object types', () => {
      expect(isSimulationFrame('string')).toBe(false);
      expect(isSimulationFrame(123)).toBe(false);
      expect(isSimulationFrame([])).toBe(false);
    });
  });

  describe('adaptSimulationFrame', () => {
    it('should convert empty agents to empty creatures', () => {
      const frame: SimulationFrame = {
        tick: 100,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.tick).toBe(100);
      expect(result.creatures).toEqual([]);
      expect(result.server_time).toBe(new Date('2025-01-05T21:00:00.000Z').getTime());
    });

    it('should convert single agent to creature', () => {
      const frame: SimulationFrame = {
        tick: 42,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 1, x: 10.5, y: 20.3, vx: 1.2, vy: -0.5, rotation: 1.57 },
        ],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.tick).toBe(42);
      expect(result.creatures).toHaveLength(1);
      expect(result.creatures[0]).toEqual({
        id: 1,
        x: 10.5,
        y: 20.3,
        rotation: 1.57,
        width: 10,
        height: 10,
      });
    });

    it('should convert multiple agents to creatures', () => {
      const frame: SimulationFrame = {
        tick: 999,
        timestamp: '2025-01-05T21:30:00.000Z',
        agents: [
          { id: 1, x: 10, y: 20, vx: 1, vy: 2, rotation: 0 },
          { id: 2, x: 30, y: 40, vx: -1, vy: 1, rotation: 3.14 },
          { id: 3, x: 50, y: 60, vx: 0, vy: -1, rotation: 1.5 },
        ],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.creatures).toHaveLength(3);
      expect(result.creatures[0].id).toBe(1);
      expect(result.creatures[1].id).toBe(2);
      expect(result.creatures[2].id).toBe(3);
    });

    it('should set default width and height for all creatures', () => {
      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 1, x: 0, y: 0, vx: 0, vy: 0, rotation: 0 },
          { id: 2, x: 10, y: 10, vx: 1, vy: 1, rotation: 1 },
        ],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.creatures[0].width).toBe(10);
      expect(result.creatures[0].height).toBe(10);
      expect(result.creatures[1].width).toBe(10);
      expect(result.creatures[1].height).toBe(10);
    });

    it('should correctly parse ISO 8601 timestamp to server_time', () => {
      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      };

      const result = adaptSimulationFrame(frame);
      const expectedTime = new Date('2025-01-05T21:00:00.000Z').getTime();

      expect(result.server_time).toBe(expectedTime);
    });

    it('should preserve agent position and rotation', () => {
      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 42, x: 123.456, y: 789.012, vx: 5, vy: -3, rotation: 2.5 },
        ],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.creatures[0].x).toBe(123.456);
      expect(result.creatures[0].y).toBe(789.012);
      expect(result.creatures[0].rotation).toBe(2.5);
    });

    it('should handle large agent IDs', () => {
      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 999999, x: 0, y: 0, vx: 0, vy: 0, rotation: 0 },
        ],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.creatures[0].id).toBe(999999);
    });

    it('should handle negative positions', () => {
      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 1, x: -50, y: -100, vx: 0, vy: 0, rotation: 0 },
        ],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.creatures[0].x).toBe(-50);
      expect(result.creatures[0].y).toBe(-100);
    });

    it('should handle rotation values from 0 to 2π', () => {
      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 1, x: 0, y: 0, vx: 0, vy: 0, rotation: 0 },
          { id: 2, x: 0, y: 0, vx: 0, vy: 0, rotation: Math.PI },
          { id: 3, x: 0, y: 0, vx: 0, vy: 0, rotation: 2 * Math.PI },
        ],
      };

      const result = adaptSimulationFrame(frame);

      expect(result.creatures[0].rotation).toBe(0);
      expect(result.creatures[1].rotation).toBeCloseTo(Math.PI);
      expect(result.creatures[2].rotation).toBeCloseTo(2 * Math.PI);
    });
  });
});
