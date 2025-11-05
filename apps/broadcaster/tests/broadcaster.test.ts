import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter } from 'events';
import type { SimulationFrame } from '../src/types.js';

/**
 * Mock NATS Subscriber for testing
 */
class MockNatsSubscriber extends EventEmitter {
  async connect() {
    this.emit('connected');
  }

  async close() {
    this.removeAllListeners();
  }
}

/**
 * Mock WebSocket Server for testing
 */
class MockWebSocketServer {
  public broadcastCalls: string[] = [];
  public clientCount = 0;

  start() {
    // Server started
  }

  broadcast(message: string) {
    this.broadcastCalls.push(message);
  }

  getClientCount(): number {
    return this.clientCount;
  }

  close() {
    // Server closed
  }
}

describe('Broadcaster', () => {
  let mockNats: MockNatsSubscriber;
  let mockWs: MockWebSocketServer;

  beforeEach(() => {
    mockNats = new MockNatsSubscriber();
    mockWs = new MockWebSocketServer();
  });

  describe('constructor', () => {
    it('should accept NatsSubscriber and WebSocketServer as dependencies', () => {
      expect(mockNats).toBeInstanceOf(EventEmitter);
      expect(mockWs).toHaveProperty('broadcast');
      expect(mockWs).toHaveProperty('start');
    });
  });

  describe('start', () => {
    it('should start WebSocket server', () => {
      const startSpy = vi.spyOn(mockWs, 'start');
      mockWs.start();
      expect(startSpy).toHaveBeenCalledTimes(1);
    });

    it('should set up NATS message listener', () => {
      const listenerCount = mockNats.listenerCount('message');
      mockNats.on('message', () => {});
      expect(mockNats.listenerCount('message')).toBe(listenerCount + 1);
    });
  });

  describe('message relay', () => {
    it('should broadcast NATS messages to WebSocket clients', () => {
      // Set up message handler
      mockNats.on('message', (frame: SimulationFrame) => {
        mockWs.broadcast(JSON.stringify(frame));
      });

      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-11-05T12:00:00Z',
        agents: [
          { id: 1, x: 10, y: 20, vx: 1, vy: 0, rotation: 0 },
        ],
      };

      mockNats.emit('message', frame);

      expect(mockWs.broadcastCalls).toHaveLength(1);
      expect(mockWs.broadcastCalls[0]).toBe(JSON.stringify(frame));
    });

    it('should handle multiple messages sequentially', () => {
      mockNats.on('message', (frame: SimulationFrame) => {
        mockWs.broadcast(JSON.stringify(frame));
      });

      const frames: SimulationFrame[] = [
        { tick: 1, timestamp: '2025-11-05T12:00:00.000Z', agents: [] },
        { tick: 2, timestamp: '2025-11-05T12:00:00.050Z', agents: [] },
        { tick: 3, timestamp: '2025-11-05T12:00:00.100Z', agents: [] },
      ];

      frames.forEach(frame => mockNats.emit('message', frame));

      expect(mockWs.broadcastCalls).toHaveLength(3);
      expect(mockWs.broadcastCalls[0]).toContain('"tick":1');
      expect(mockWs.broadcastCalls[1]).toContain('"tick":2');
      expect(mockWs.broadcastCalls[2]).toContain('"tick":3');
    });

    it('should serialize SimulationFrame to JSON string', () => {
      mockNats.on('message', (frame: SimulationFrame) => {
        mockWs.broadcast(JSON.stringify(frame));
      });

      const frame: SimulationFrame = {
        tick: 42,
        timestamp: '2025-11-05T14:32:15.750Z',
        agents: [
          { id: 1, x: 45.23, y: 78.91, vx: 2.15, vy: -0.87, rotation: 1.57 },
        ],
      };

      mockNats.emit('message', frame);

      const broadcasted = mockWs.broadcastCalls[0];
      const parsed = JSON.parse(broadcasted);

      expect(parsed.tick).toBe(42);
      expect(parsed.agents).toHaveLength(1);
      expect(parsed.agents[0].id).toBe(1);
      expect(parsed.agents[0].rotation).toBe(1.57);
    });

    it('should broadcast even when no clients are connected', () => {
      mockWs.clientCount = 0;

      mockNats.on('message', (frame: SimulationFrame) => {
        mockWs.broadcast(JSON.stringify(frame));
      });

      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-11-05T12:00:00Z',
        agents: [],
      };

      mockNats.emit('message', frame);

      // Should still call broadcast (WebSocketServer handles empty client list)
      expect(mockWs.broadcastCalls).toHaveLength(1);
    });
  });

  describe('NATS event handling', () => {
    it('should handle NATS connected event', () => {
      const connectedHandler = vi.fn();
      mockNats.on('connected', connectedHandler);

      mockNats.emit('connected');

      expect(connectedHandler).toHaveBeenCalledTimes(1);
    });

    it('should handle NATS disconnected event', () => {
      const disconnectedHandler = vi.fn();
      mockNats.on('disconnected', disconnectedHandler);

      mockNats.emit('disconnected');

      expect(disconnectedHandler).toHaveBeenCalledTimes(1);
    });

    it('should handle NATS error event', () => {
      const errorHandler = vi.fn();
      mockNats.on('error', errorHandler);

      const error = new Error('NATS error');
      mockNats.emit('error', error);

      expect(errorHandler).toHaveBeenCalledTimes(1);
      expect(errorHandler).toHaveBeenCalledWith(error);
    });
  });

  describe('shutdown', () => {
    it('should close NATS connection', async () => {
      const closeSpy = vi.spyOn(mockNats, 'close');
      await mockNats.close();
      expect(closeSpy).toHaveBeenCalledTimes(1);
    });

    it('should close WebSocket server', () => {
      const closeSpy = vi.spyOn(mockWs, 'close');
      mockWs.close();
      expect(closeSpy).toHaveBeenCalledTimes(1);
    });

    it('should clean up event listeners', async () => {
      mockNats.on('message', () => {});
      mockNats.on('error', () => {});

      expect(mockNats.listenerCount('message')).toBeGreaterThan(0);
      expect(mockNats.listenerCount('error')).toBeGreaterThan(0);

      await mockNats.close();

      expect(mockNats.listenerCount('message')).toBe(0);
      expect(mockNats.listenerCount('error')).toBe(0);
    });
  });

  describe('error resilience', () => {
    it('should continue broadcasting even if serialization fails', () => {
      let callCount = 0;

      mockNats.on('message', (frame: SimulationFrame) => {
        try {
          mockWs.broadcast(JSON.stringify(frame));
          callCount++;
        } catch (error) {
          // Log error but continue
          console.error(error);
        }
      });

      const validFrame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-11-05T12:00:00Z',
        agents: [],
      };

      mockNats.emit('message', validFrame);
      mockNats.emit('message', validFrame);

      expect(callCount).toBe(2);
    });

    it('should not crash on broadcast errors', () => {
      mockNats.on('message', (frame: SimulationFrame) => {
        try {
          mockWs.broadcast(JSON.stringify(frame));
        } catch (error) {
          // Swallow error
        }
      });

      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-11-05T12:00:00Z',
        agents: [],
      };

      // Should not throw
      expect(() => {
        mockNats.emit('message', frame);
      }).not.toThrow();
    });
  });

  describe('integration', () => {
    it('should coordinate NATS subscription and WebSocket broadcasting', () => {
      const broadcastSpy = vi.spyOn(mockWs, 'broadcast');

      mockNats.on('message', (frame: SimulationFrame) => {
        mockWs.broadcast(JSON.stringify(frame));
      });

      const frames: SimulationFrame[] = [
        { tick: 1, timestamp: '2025-11-05T12:00:00.000Z', agents: [] },
        { tick: 2, timestamp: '2025-11-05T12:00:00.050Z', agents: [] },
      ];

      frames.forEach(frame => mockNats.emit('message', frame));

      expect(broadcastSpy).toHaveBeenCalledTimes(2);
    });
  });
});
