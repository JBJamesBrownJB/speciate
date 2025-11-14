import { describe, it, expect } from 'vitest';
import {
  isSimulationStateMessage,
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
});
