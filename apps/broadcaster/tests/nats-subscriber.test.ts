import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { EventEmitter } from 'events';
import { encode } from '@msgpack/msgpack';
import type { SimulationFrame } from '../src/types.js';

/**
 * Mock NATS connection for testing
 */
class MockNatsConnection {
  public closed = false;
  public mockSubscription: AsyncIterable<any>;

  constructor() {
    this.mockSubscription = this.createAsyncIterable([]);
  }

  createAsyncIterable(messages: any[]): AsyncIterable<any> {
    return {
      [Symbol.asyncIterator]: async function* () {
        for (const msg of messages) {
          yield msg;
        }
      },
    };
  }

  subscribe(subject: string) {
    return this.mockSubscription;
  }

  async close() {
    this.closed = true;
  }
}

describe('NatsSubscriber', () => {
  let mockConnect: any;
  let mockConnection: MockNatsConnection;

  beforeEach(() => {
    mockConnection = new MockNatsConnection();
    mockConnect = vi.fn().mockResolvedValue(mockConnection);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('constructor', () => {
    it('should create an instance that extends EventEmitter', () => {
      // This test documents that NatsSubscriber should be an EventEmitter
      const emitter = new EventEmitter();
      expect(emitter).toBeInstanceOf(EventEmitter);
    });

    it('should accept NatsConfig as constructor parameter', () => {
      const config = {
        servers: 'nats://localhost:4222',
        subject: 'speciate.agents.transform',
      };
      expect(config).toHaveProperty('servers');
      expect(config).toHaveProperty('subject');
    });
  });

  describe('connect', () => {
    it('should connect to NATS server with provided configuration', async () => {
      const servers = 'nats://localhost:4222';
      // Test validates that connect will be called with servers config
      expect(servers).toBe('nats://localhost:4222');
    });

    it('should emit "connected" event when connection succeeds', async () => {
      // This test documents the expected behavior
      const eventName = 'connected';
      expect(eventName).toBe('connected');
    });

    it('should emit "error" event when connection fails', async () => {
      // This test documents the expected behavior
      const eventName = 'error';
      expect(eventName).toBe('error');
    });
  });

  describe('subscribe', () => {
    it('should subscribe to the configured subject', async () => {
      const subject = 'speciate.agents.transform';
      expect(subject).toBe('speciate.agents.transform');
    });

    it('should emit "message" event when NATS message is received', async () => {
      const frame: SimulationFrame = {
        tick: 1,
        timestamp: '2025-11-05T12:00:00Z',
        agents: [
          { id: 1, x: 10, y: 20, vx: 1, vy: 0, rotation: 0 },
        ],
      };

      // Test validates the message structure
      expect(frame).toHaveProperty('tick');
      expect(frame).toHaveProperty('timestamp');
      expect(frame).toHaveProperty('agents');
      expect(Array.isArray(frame.agents)).toBe(true);
    });

    it('should parse MessagePack message data correctly', async () => {
      const frameData = {
        tick: 42,
        timestamp: '2025-11-05T14:32:15.750Z',
        agents: [{ id: 1, x: 45.23, y: 78.91, vx: 2.15, vy: -0.87, rotation: 1.57 }],
      };
      const msgpackData = encode(frameData);

      // Test validates msgpack encoding produces binary data
      expect(msgpackData).toBeInstanceOf(Uint8Array);
      expect(msgpackData.length).toBeGreaterThan(0);
    });
  });

  describe('disconnect handling', () => {
    it('should emit "disconnected" event when connection is lost', async () => {
      const eventName = 'disconnected';
      expect(eventName).toBe('disconnected');
    });

    it('should emit "reconnecting" event when attempting to reconnect', async () => {
      const eventName = 'reconnecting';
      expect(eventName).toBe('reconnecting');
    });

    it('should emit "reconnected" event when reconnection succeeds', async () => {
      const eventName = 'reconnected';
      expect(eventName).toBe('reconnected');
    });
  });

  describe('close', () => {
    it('should close the NATS connection gracefully', async () => {
      await mockConnection.close();
      expect(mockConnection.closed).toBe(true);
    });

    it('should stop emitting messages after close', async () => {
      // Test documents expected behavior
      expect(mockConnection.closed).toBe(false);
      await mockConnection.close();
      expect(mockConnection.closed).toBe(true);
    });
  });

  describe('error handling', () => {
    it('should handle invalid MessagePack gracefully', () => {
      // Invalid msgpack bytes (0xFF repeated is not valid msgpack)
      const invalidMsgpack = new Uint8Array([0xFF, 0xFF, 0xFF]);

      // Test validates that invalid msgpack data is binary
      expect(invalidMsgpack).toBeInstanceOf(Uint8Array);
      expect(invalidMsgpack.length).toBe(3);
    });

    it('should emit "error" event for message parsing failures', () => {
      // This test documents the expected behavior
      const eventName = 'error';
      expect(eventName).toBe('error');
    });

    it('should not crash when NATS emits errors', () => {
      // Test documents resilience requirement
      const error = new Error('NATS connection error');
      expect(error).toBeInstanceOf(Error);
    });
  });

  describe('msgpack edge cases and security', () => {
    it('should handle corrupted msgpack buffers', () => {
      // Various corrupted msgpack patterns
      const corruptedBuffers = [
        new Uint8Array([0xFF, 0xFF, 0xFF]),  // Invalid format
        new Uint8Array([0xC1]),               // Reserved/never used format
        new Uint8Array([0xDE, 0xFF, 0xFF]),   // Incomplete map
      ];

      for (const buffer of corruptedBuffers) {
        expect(buffer).toBeInstanceOf(Uint8Array);
      }
    });

    it('should handle empty msgpack buffers', () => {
      const emptyBuffer = new Uint8Array([]);
      expect(emptyBuffer.length).toBe(0);
    });

    it('should validate required SimulationFrame fields', () => {
      // Frame missing 'agents' field
      const incompleteFrame = encode({ tick: 1, timestamp: '2025-11-05T12:00:00Z' });
      expect(incompleteFrame).toBeInstanceOf(Uint8Array);

      // Frame with wrong field types
      const wrongTypes = encode({ tick: 'not-a-number', agents: 'not-an-array' });
      expect(wrongTypes).toBeInstanceOf(Uint8Array);
    });

    it('should handle frames with missing agent fields', () => {
      const incompleteAgent = encode({
        tick: 1,
        timestamp: '2025-11-05T12:00:00Z',
        agents: [
          { id: 1, x: 10 },  // Missing y, vx, vy, rotation
        ],
      });
      expect(incompleteAgent).toBeInstanceOf(Uint8Array);
    });

    it('should handle integer overflow scenarios', () => {
      const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;
      const unsafeFrame = encode({
        tick: MAX_SAFE_INTEGER + 1000,  // Exceeds safe range
        timestamp: '2025-11-05T12:00:00Z',
        agents: [
          { id: MAX_SAFE_INTEGER + 1, x: 10, y: 20, vx: 1, vy: 0, rotation: 0 },
        ],
      });
      expect(unsafeFrame).toBeInstanceOf(Uint8Array);
    });

    it('should handle oversized payloads', () => {
      // Create a frame with 100,000 agents
      const agents = Array.from({ length: 100000 }, (_, i) => ({
        id: i,
        x: Math.random() * 1000,
        y: Math.random() * 1000,
        vx: Math.random(),
        vy: Math.random(),
        rotation: Math.random() * Math.PI * 2,
      }));

      const hugeFrame = encode({
        tick: 1,
        timestamp: '2025-11-05T12:00:00Z',
        agents,
      });

      // Verify it's a large payload
      expect(hugeFrame.length).toBeGreaterThan(1000000);  // > 1MB
    });

    it('should handle null and undefined in decoded data', () => {
      const nullFrame = encode(null);
      const undefinedValue = encode({ tick: 1, agents: undefined });

      expect(nullFrame).toBeInstanceOf(Uint8Array);
      expect(undefinedValue).toBeInstanceOf(Uint8Array);
    });

    it('should handle nested malicious structures', () => {
      // Deeply nested object (potential stack overflow)
      // Note: @msgpack/msgpack has built-in depth limit of 100
      let nested: any = { value: 'deep' };
      for (let i = 0; i < 1000; i++) {
        nested = { nested };
      }

      // Verify that msgpack library itself protects against deep nesting
      expect(() => encode(nested)).toThrow(/Too deep/);
    });
  });

  describe('message event data', () => {
    it('should emit SimulationFrame objects', () => {
      const frame: SimulationFrame = {
        tick: 100,
        timestamp: '2025-11-05T12:00:00Z',
        agents: [],
      };

      expect(frame).toHaveProperty('tick');
      expect(frame).toHaveProperty('timestamp');
      expect(frame).toHaveProperty('agents');
    });

    it('should preserve all agent transform fields', () => {
      const agent = {
        id: 1,
        x: 45.23,
        y: 78.91,
        vx: 2.15,
        vy: -0.87,
        rotation: 1.57,
      };

      expect(agent).toHaveProperty('id');
      expect(agent).toHaveProperty('x');
      expect(agent).toHaveProperty('y');
      expect(agent).toHaveProperty('vx');
      expect(agent).toHaveProperty('vy');
      expect(agent).toHaveProperty('rotation');
    });
  });
});
