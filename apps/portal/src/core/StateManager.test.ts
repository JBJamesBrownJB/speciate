import { describe, it, expect, beforeEach } from 'vitest';
import { StateManager } from './StateManager';
import type { SimulationStateMessage } from '@/types/messages';

describe('StateManager', () => {
  let stateManager: StateManager;

  beforeEach(() => {
    stateManager = new StateManager();
  });

  describe('getCurrentTick', () => {
    it('should return 0 initially', () => {
      expect(stateManager.getCurrentTick()).toBe(0);
    });

    it('should return the last updated tick', () => {
      const message: SimulationStateMessage = {
        tick: 100,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message);

      expect(stateManager.getCurrentTick()).toBe(100);
    });
  });

  describe('updateFromServer', () => {
    it('should update tick from message', () => {
      const message: SimulationStateMessage = {
        tick: 42,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message);

      expect(stateManager.getCurrentTick()).toBe(42);
    });

    it('should update tick with multiple messages', () => {
      const message1: SimulationStateMessage = {
        tick: 100,
        server_time: Date.now(),
        creatures: [],
      };

      const message2: SimulationStateMessage = {
        tick: 200,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message1);
      expect(stateManager.getCurrentTick()).toBe(100);

      stateManager.updateFromServer(message2);
      expect(stateManager.getCurrentTick()).toBe(200);
    });

    it('should handle tick 0', () => {
      const message: SimulationStateMessage = {
        tick: 0,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message);

      expect(stateManager.getCurrentTick()).toBe(0);
    });

    it('should handle large tick numbers', () => {
      const message: SimulationStateMessage = {
        tick: 999999999,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message);

      expect(stateManager.getCurrentTick()).toBe(999999999);
    });

    it('should handle messages with creatures', () => {
      const message: SimulationStateMessage = {
        tick: 50,
        server_time: Date.now(),
        creatures: [
          { id: 1, x: 10, y: 20, rotation: 0.5, size: 10 },
          { id: 2, x: 30, y: 40, rotation: 1.0, size: 10 },
        ],
      };

      stateManager.updateFromServer(message);

      expect(stateManager.getCurrentTick()).toBe(50);
    });

    it('should update tick even if previous tick was higher (handles out-of-order messages)', () => {
      const message1: SimulationStateMessage = {
        tick: 200,
        server_time: Date.now(),
        creatures: [],
      };

      const message2: SimulationStateMessage = {
        tick: 100,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message1);
      expect(stateManager.getCurrentTick()).toBe(200);

      stateManager.updateFromServer(message2);
      // StateManager doesn't validate order, just tracks latest
      expect(stateManager.getCurrentTick()).toBe(100);
    });
  });

  describe('clear', () => {
    it('should reset tick to 0', () => {
      const message: SimulationStateMessage = {
        tick: 500,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message);
      expect(stateManager.getCurrentTick()).toBe(500);

      stateManager.clear();

      expect(stateManager.getCurrentTick()).toBe(0);
    });

    it('should allow updates after clear', () => {
      const message1: SimulationStateMessage = {
        tick: 100,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message1);
      stateManager.clear();

      const message2: SimulationStateMessage = {
        tick: 200,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message2);

      expect(stateManager.getCurrentTick()).toBe(200);
    });

    it('should be idempotent (multiple clears are safe)', () => {
      const message: SimulationStateMessage = {
        tick: 100,
        server_time: Date.now(),
        creatures: [],
      };

      stateManager.updateFromServer(message);

      stateManager.clear();
      stateManager.clear();
      stateManager.clear();

      expect(stateManager.getCurrentTick()).toBe(0);
    });
  });

  describe('Integration scenarios', () => {
    it('should track tick progression through simulation lifecycle', () => {
      // Simulation starts
      expect(stateManager.getCurrentTick()).toBe(0);

      // First update
      stateManager.updateFromServer({
        tick: 1,
        server_time: Date.now(),
        creatures: [],
      });
      expect(stateManager.getCurrentTick()).toBe(1);

      // Simulation runs
      for (let i = 2; i <= 100; i++) {
        stateManager.updateFromServer({
          tick: i,
          server_time: Date.now(),
          creatures: [],
        });
      }
      expect(stateManager.getCurrentTick()).toBe(100);

      // Connection lost, state cleared
      stateManager.clear();
      expect(stateManager.getCurrentTick()).toBe(0);

      // Reconnected, simulation continues
      stateManager.updateFromServer({
        tick: 101,
        server_time: Date.now(),
        creatures: [],
      });
      expect(stateManager.getCurrentTick()).toBe(101);
    });

    it('should handle rapid updates', () => {
      for (let i = 0; i < 1000; i++) {
        stateManager.updateFromServer({
          tick: i,
          server_time: Date.now(),
          creatures: [],
        });
      }

      expect(stateManager.getCurrentTick()).toBe(999);
    });
  });
});
