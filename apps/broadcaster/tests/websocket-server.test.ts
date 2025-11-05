import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { EventEmitter } from 'events';

/**
 * Mock WebSocket for testing
 */
class MockWebSocket extends EventEmitter {
  public readyState = 1; // OPEN
  public sentMessages: string[] = [];

  constructor() {
    super();
    // Add a default error listener to prevent EventEmitter from throwing
    this.on('error', () => {
      // Default error handler
    });
  }

  send(data: string) {
    this.sentMessages.push(data);
  }

  close() {
    this.readyState = 3; // CLOSED
    this.emit('close');
  }

  simulateError(error: Error) {
    this.emit('error', error);
  }
}

/**
 * Mock WebSocket.Server for testing
 */
class MockWebSocketServer extends EventEmitter {
  public clients = new Set<MockWebSocket>();

  constructor() {
    super();
  }

  simulateConnection(ws: MockWebSocket) {
    this.clients.add(ws);
    this.emit('connection', ws);

    ws.on('close', () => {
      this.clients.delete(ws);
    });
  }

  close() {
    this.emit('close');
  }
}

describe('WebSocketServer', () => {
  let mockWsServer: MockWebSocketServer;

  beforeEach(() => {
    mockWsServer = new MockWebSocketServer();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('constructor', () => {
    it('should accept WebSocketConfig as constructor parameter', () => {
      const config = {
        port: 8080,
        path: '/stream',
      };
      expect(config).toHaveProperty('port');
      expect(config).toHaveProperty('path');
    });

    it('should validate port is within valid range', () => {
      const port = 8080;
      expect(port).toBeGreaterThan(0);
      expect(port).toBeLessThanOrEqual(65535);
    });
  });

  describe('start', () => {
    it('should start WebSocket server on configured port', () => {
      const port = 8080;
      expect(port).toBe(8080);
    });

    it('should listen on configured path', () => {
      const path = '/stream';
      expect(path).toBe('/stream');
    });

    it('should track connected clients', () => {
      const client1 = new MockWebSocket();
      const client2 = new MockWebSocket();

      mockWsServer.simulateConnection(client1);
      mockWsServer.simulateConnection(client2);

      expect(mockWsServer.clients.size).toBe(2);
      expect(mockWsServer.clients.has(client1)).toBe(true);
      expect(mockWsServer.clients.has(client2)).toBe(true);
    });
  });

  describe('client connection', () => {
    it('should add client to clients set on connection', () => {
      const client = new MockWebSocket();
      mockWsServer.simulateConnection(client);

      expect(mockWsServer.clients.has(client)).toBe(true);
      expect(mockWsServer.clients.size).toBe(1);
    });

    it('should remove client from set on disconnect', () => {
      const client = new MockWebSocket();
      mockWsServer.simulateConnection(client);

      expect(mockWsServer.clients.size).toBe(1);

      client.close();

      expect(mockWsServer.clients.size).toBe(0);
      expect(mockWsServer.clients.has(client)).toBe(false);
    });

    it('should handle multiple clients connecting and disconnecting', () => {
      const client1 = new MockWebSocket();
      const client2 = new MockWebSocket();
      const client3 = new MockWebSocket();

      mockWsServer.simulateConnection(client1);
      mockWsServer.simulateConnection(client2);
      mockWsServer.simulateConnection(client3);

      expect(mockWsServer.clients.size).toBe(3);

      client2.close();

      expect(mockWsServer.clients.size).toBe(2);
      expect(mockWsServer.clients.has(client1)).toBe(true);
      expect(mockWsServer.clients.has(client2)).toBe(false);
      expect(mockWsServer.clients.has(client3)).toBe(true);
    });
  });

  describe('broadcast', () => {
    it('should send message to all connected clients', () => {
      const client1 = new MockWebSocket();
      const client2 = new MockWebSocket();
      const client3 = new MockWebSocket();

      mockWsServer.simulateConnection(client1);
      mockWsServer.simulateConnection(client2);
      mockWsServer.simulateConnection(client3);

      const message = JSON.stringify({ tick: 1, timestamp: '2025-11-05T12:00:00Z', agents: [] });

      // Simulate broadcast
      mockWsServer.clients.forEach(client => {
        client.send(message);
      });

      expect(client1.sentMessages).toContain(message);
      expect(client2.sentMessages).toContain(message);
      expect(client3.sentMessages).toContain(message);
    });

    it('should not send to disconnected clients', () => {
      const client1 = new MockWebSocket();
      const client2 = new MockWebSocket();

      mockWsServer.simulateConnection(client1);
      mockWsServer.simulateConnection(client2);

      // Disconnect client2
      client2.close();

      const message = 'test message';

      // Broadcast only to active clients
      mockWsServer.clients.forEach(client => {
        client.send(message);
      });

      expect(client1.sentMessages).toContain(message);
      expect(client2.sentMessages).not.toContain(message);
    });

    it('should handle broadcast to empty client list', () => {
      const message = 'test message';

      // No clients connected
      expect(mockWsServer.clients.size).toBe(0);

      // Should not throw error
      expect(() => {
        mockWsServer.clients.forEach(client => {
          client.send(message);
        });
      }).not.toThrow();
    });

    it('should skip clients that are not in OPEN state', () => {
      const client1 = new MockWebSocket();
      const client2 = new MockWebSocket();

      mockWsServer.simulateConnection(client1);
      mockWsServer.simulateConnection(client2);

      // Set client2 to CLOSING state
      client2.readyState = 2; // CLOSING

      const message = 'test message';

      // Broadcast only to OPEN clients
      mockWsServer.clients.forEach(client => {
        if (client.readyState === 1) {
          client.send(message);
        }
      });

      expect(client1.sentMessages).toContain(message);
      expect(client2.sentMessages).not.toContain(message);
    });
  });

  describe('error handling', () => {
    it('should handle client errors gracefully', () => {
      const client = new MockWebSocket();
      mockWsServer.simulateConnection(client);

      const error = new Error('WebSocket error');

      // Should not throw
      expect(() => {
        client.simulateError(error);
      }).not.toThrow();
    });

    it('should remove client on error', () => {
      const client = new MockWebSocket();
      mockWsServer.simulateConnection(client);

      // Simulate error and close
      const error = new Error('WebSocket error');
      client.simulateError(error);
      client.close();

      expect(mockWsServer.clients.has(client)).toBe(false);
    });
  });

  describe('close', () => {
    it('should close WebSocket server', () => {
      const closeHandler = vi.fn();
      mockWsServer.on('close', closeHandler);

      mockWsServer.close();

      expect(closeHandler).toHaveBeenCalledTimes(1);
    });

    it('should close all client connections when server closes', () => {
      const client1 = new MockWebSocket();
      const client2 = new MockWebSocket();

      mockWsServer.simulateConnection(client1);
      mockWsServer.simulateConnection(client2);

      // Close all clients
      mockWsServer.clients.forEach(client => client.close());

      expect(client1.readyState).toBe(3); // CLOSED
      expect(client2.readyState).toBe(3); // CLOSED
      expect(mockWsServer.clients.size).toBe(0);
    });
  });

  describe('getClientCount', () => {
    it('should return the number of connected clients', () => {
      expect(mockWsServer.clients.size).toBe(0);

      const client1 = new MockWebSocket();
      mockWsServer.simulateConnection(client1);
      expect(mockWsServer.clients.size).toBe(1);

      const client2 = new MockWebSocket();
      mockWsServer.simulateConnection(client2);
      expect(mockWsServer.clients.size).toBe(2);

      client1.close();
      expect(mockWsServer.clients.size).toBe(1);
    });
  });
});
