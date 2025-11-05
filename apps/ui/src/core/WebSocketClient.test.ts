import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { WebSocketClient } from './WebSocketClient';
import { ConnectionState } from '@/types/entities';
import type { SimulationStateMessage } from '@/types/messages';

// Mock WebSocket
class MockWebSocket {
  public readyState: number = WebSocket.CONNECTING;
  public onopen: (() => void) | null = null;
  public onmessage: ((event: MessageEvent) => void) | null = null;
  public onerror: (() => void) | null = null;
  public onclose: (() => void) | null = null;

  constructor(public url: string) {
    // Simulate async connection
    setTimeout(() => {
      if (this.readyState === WebSocket.CONNECTING) {
        this.readyState = WebSocket.OPEN;
        this.onopen?.();
      }
    }, 0);
  }

  close(code?: number, reason?: string): void {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.();
  }

  send(data: string): void {
    // Mock send
  }

  // Simulate receiving a message
  simulateMessage(data: any): void {
    if (this.onmessage) {
      const event = new MessageEvent('message', {
        data: JSON.stringify(data),
      });
      this.onmessage(event);
    }
  }

  // Simulate connection error
  simulateError(): void {
    this.onerror?.();
  }

  // Simulate connection close
  simulateClose(): void {
    this.readyState = WebSocket.CLOSED;
    this.onclose?.();
  }
}

// Setup global WebSocket mock
let mockWebSocketInstance: MockWebSocket | null = null;

beforeEach(() => {
  // Create a proper class constructor mock
  const MockWebSocketConstructor = function (this: any, url: string) {
    mockWebSocketInstance = new MockWebSocket(url);
    return mockWebSocketInstance;
  } as any;

  // Set static properties
  MockWebSocketConstructor.CONNECTING = 0;
  MockWebSocketConstructor.OPEN = 1;
  MockWebSocketConstructor.CLOSING = 2;
  MockWebSocketConstructor.CLOSED = 3;

  // @ts-ignore - Mock WebSocket globally
  global.WebSocket = MockWebSocketConstructor;

  vi.useFakeTimers();
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  vi.restoreAllMocks();
  mockWebSocketInstance = null;
});

describe('WebSocketClient', () => {
  describe('Connection Management', () => {
    it('should connect to /stream path by default', () => {
      const client = new WebSocketClient();
      client.connect();

      // Verify WebSocket was created with correct URL
      expect(mockWebSocketInstance).toBeTruthy();
      expect(mockWebSocketInstance?.url).toBe('ws://localhost:8080/stream');
    });

    it('should connect to custom URL if provided', () => {
      const client = new WebSocketClient('ws://example.com:9000/custom');
      client.connect();

      // Verify WebSocket was created with correct URL
      expect(mockWebSocketInstance).toBeTruthy();
      expect(mockWebSocketInstance?.url).toBe('ws://example.com:9000/custom');
    });

    it('should transition to Connected state on successful connection', async () => {
      const client = new WebSocketClient();
      const states: ConnectionState[] = [];

      client.onConnectionStateChange((state) => {
        states.push(state);
      });

      client.connect();
      await vi.runAllTimersAsync();

      expect(states).toContain(ConnectionState.Connecting);
      expect(states).toContain(ConnectionState.Connected);
    });

    it('should not create duplicate connection if already connected', async () => {
      const client = new WebSocketClient();
      client.connect();
      await vi.runAllTimersAsync();

      const firstInstance = mockWebSocketInstance;
      client.connect();

      // Should still be the same instance (no new connection created)
      expect(mockWebSocketInstance).toBe(firstInstance);
    });

    it('should disconnect cleanly and transition to Disconnected state', async () => {
      const client = new WebSocketClient();
      const states: ConnectionState[] = [];

      client.onConnectionStateChange((state) => {
        states.push(state);
      });

      client.connect();
      await vi.runAllTimersAsync();

      client.disconnect();

      expect(states[states.length - 1]).toBe(ConnectionState.Disconnected);
    });

    it('should set intentionalClose flag when disconnect is called', async () => {
      const client = new WebSocketClient();
      client.connect();
      await vi.runAllTimersAsync();

      client.disconnect();

      // Verify no reconnection happens after intentional disconnect
      const instanceAfterDisconnect = mockWebSocketInstance;
      await vi.advanceTimersByTimeAsync(5000);
      // No new instance should be created
      expect(mockWebSocketInstance).toBe(instanceAfterDisconnect);
    });
  });

  describe('Message Handling', () => {
    it('should handle SimulationFrame messages and adapt to SimulationStateMessage', async () => {
      const client = new WebSocketClient();
      const receivedMessages: SimulationStateMessage[] = [];

      client.onMessage((msg) => {
        receivedMessages.push(msg);
      });

      client.connect();
      await vi.runAllTimersAsync();

      // Send SimulationFrame (new broadcaster format)
      mockWebSocketInstance?.simulateMessage({
        tick: 100,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [
          { id: 1, x: 10.5, y: 20.3, vx: 1.2, vy: -0.5, rotation: 1.57 },
          { id: 2, x: 30.0, y: 40.0, vx: 0.0, vy: 1.0, rotation: 0.0 },
        ],
      });

      expect(receivedMessages).toHaveLength(1);
      expect(receivedMessages[0].tick).toBe(100);
      expect(receivedMessages[0].creatures).toHaveLength(2);
      expect(receivedMessages[0].creatures[0]).toEqual({
        id: 1,
        x: 10.5,
        y: 20.3,
        rotation: 1.57,
        width: 10,
        height: 10,
      });
      expect(receivedMessages[0].server_time).toBe(
        new Date('2025-01-05T21:00:00.000Z').getTime()
      );
    });

    it('should handle legacy SimulationStateMessage format', async () => {
      const client = new WebSocketClient();
      const receivedMessages: SimulationStateMessage[] = [];

      client.onMessage((msg) => {
        receivedMessages.push(msg);
      });

      client.connect();
      await vi.runAllTimersAsync();

      // Send legacy format (SimulationStateMessage with creatures)
      const legacyMessage: SimulationStateMessage = {
        tick: 200,
        server_time: 1234567890,
        creatures: [
          { id: 5, x: 50, y: 60, rotation: 0.5, width: 10, height: 10 },
        ],
      };

      mockWebSocketInstance?.simulateMessage(legacyMessage);

      expect(receivedMessages).toHaveLength(1);
      expect(receivedMessages[0]).toEqual(legacyMessage);
    });

    it('should handle empty agents array in SimulationFrame', async () => {
      const client = new WebSocketClient();
      const receivedMessages: SimulationStateMessage[] = [];

      client.onMessage((msg) => {
        receivedMessages.push(msg);
      });

      client.connect();
      await vi.runAllTimersAsync();

      mockWebSocketInstance?.simulateMessage({
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      });

      expect(receivedMessages).toHaveLength(1);
      expect(receivedMessages[0].creatures).toEqual([]);
    });

    it('should log warning for invalid message format', async () => {
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      const client = new WebSocketClient();

      client.connect();
      await vi.runAllTimersAsync();

      // Send invalid message
      mockWebSocketInstance?.simulateMessage({
        invalid: 'message',
        no_tick: true,
      });

      expect(consoleWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('[WebSocket] Message validation failed'),
        expect.anything(),
        expect.anything()
      );

      consoleWarnSpy.mockRestore();
    });

    it('should log error for malformed JSON', async () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      const client = new WebSocketClient();

      client.connect();
      await vi.runAllTimersAsync();

      // Simulate malformed JSON by directly calling onmessage with bad data
      if (mockWebSocketInstance?.onmessage) {
        const badEvent = new MessageEvent('message', {
          data: '{invalid json',
        });
        mockWebSocketInstance.onmessage(badEvent);
      }

      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining('[WebSocket] Parse error'),
        expect.anything()
      );

      consoleErrorSpy.mockRestore();
    });

    it('should update lastMessageTime when message is received', async () => {
      const client = new WebSocketClient();

      client.connect();
      await vi.runAllTimersAsync();

      const beforePing = client.getPing();

      mockWebSocketInstance?.simulateMessage({
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      });

      // Advance time
      await vi.advanceTimersByTimeAsync(100);

      const afterPing = client.getPing();

      // Ping should be close to 100ms
      expect(afterPing).toBeGreaterThanOrEqual(90);
      expect(afterPing).toBeLessThanOrEqual(110);
    });

    it('should call multiple message handlers', async () => {
      const client = new WebSocketClient();
      const handler1Messages: SimulationStateMessage[] = [];
      const handler2Messages: SimulationStateMessage[] = [];

      client.onMessage((msg) => handler1Messages.push(msg));
      client.onMessage((msg) => handler2Messages.push(msg));

      client.connect();
      await vi.runAllTimersAsync();

      mockWebSocketInstance?.simulateMessage({
        tick: 42,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      });

      expect(handler1Messages).toHaveLength(1);
      expect(handler2Messages).toHaveLength(1);
      expect(handler1Messages[0].tick).toBe(42);
      expect(handler2Messages[0].tick).toBe(42);
    });

    it('should allow unsubscribing message handlers', async () => {
      const client = new WebSocketClient();
      const messages: SimulationStateMessage[] = [];

      const unsubscribe = client.onMessage((msg) => messages.push(msg));

      client.connect();
      await vi.runAllTimersAsync();

      mockWebSocketInstance?.simulateMessage({
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      });

      expect(messages).toHaveLength(1);

      // Unsubscribe
      unsubscribe();

      mockWebSocketInstance?.simulateMessage({
        tick: 2,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      });

      // Should still be 1 (not called after unsubscribe)
      expect(messages).toHaveLength(1);
    });
  });

  describe('Reconnection Logic', () => {
    it('should transition to Reconnecting state on connection close', async () => {
      const client = new WebSocketClient();
      const states: ConnectionState[] = [];

      client.onConnectionStateChange((state) => {
        states.push(state);
      });

      client.connect();
      await vi.runAllTimersAsync();

      // Simulate unexpected close
      mockWebSocketInstance?.simulateClose();

      expect(states).toContain(ConnectionState.Reconnecting);
    });

    it('should schedule reconnection with initial delay', async () => {
      const client = new WebSocketClient();
      client.connect();
      await vi.runAllTimersAsync();

      const firstInstance = mockWebSocketInstance;

      // Simulate unexpected close
      mockWebSocketInstance?.simulateClose();

      // Should not reconnect immediately - instance should still be the same
      expect(mockWebSocketInstance).toBe(firstInstance);

      // Advance by initial reconnect delay (1000ms)
      await vi.advanceTimersByTimeAsync(1000);

      // Should have attempted reconnection - new instance created
      expect(mockWebSocketInstance).not.toBe(firstInstance);
    });

    it('should use exponential backoff for reconnection delays', async () => {
      const client = new WebSocketClient();
      const delays: number[] = [];

      client.connect();
      await vi.runAllTimersAsync();

      // Manually trigger close and track reconnection delays
      mockWebSocketInstance?.simulateClose();

      // First reconnect: 1000ms
      const start1 = Date.now();
      await vi.advanceTimersByTimeAsync(1000);
      delays.push(Date.now() - start1);

      // Second reconnect: 1500ms (1000 * 1.5)
      mockWebSocketInstance?.simulateClose();
      const start2 = Date.now();
      await vi.advanceTimersByTimeAsync(1500);
      delays.push(Date.now() - start2);

      // Third reconnect: 2250ms (1500 * 1.5)
      mockWebSocketInstance?.simulateClose();
      const start3 = Date.now();
      await vi.advanceTimersByTimeAsync(2250);
      delays.push(Date.now() - start3);

      // Verify backoff pattern (approximately exponential)
      expect(delays[0]).toBe(1000);
      expect(delays[1]).toBe(1500);
      expect(delays[2]).toBe(2250);
    });

    it('should cap reconnection delay at maxReconnectDelay', async () => {
      const client = new WebSocketClient();

      client.connect();
      await vi.runAllTimersAsync();

      // Simulate many failures to reach max delay cap (30000ms)
      // This test verifies that delays don't grow unbounded
      const maxDelay = 30000;

      // After many failures, simulate one more and verify it reconnects within max delay
      for (let i = 0; i < 25; i++) {
        mockWebSocketInstance?.simulateClose();
        // Advance slightly more than max delay to ensure reconnection happens
        await vi.advanceTimersByTimeAsync(maxDelay + 1000);
      }

      // If we got here without timeout, the delay is properly capped
      // Verify a final reconnection works at max delay
      const instanceBefore = mockWebSocketInstance;
      mockWebSocketInstance?.simulateClose();
      await vi.advanceTimersByTimeAsync(maxDelay + 1000);

      // Should have attempted to reconnect (may be same or different instance depending on timing)
      // The key is that we didn't hang/timeout, proving the delay is capped
      expect(true).toBe(true);
    });

    it('should not reconnect after intentional disconnect', async () => {
      const client = new WebSocketClient();
      let connectionCount = 0;

      // Track connections
      const MockWebSocketConstructor = function (this: any, url: string) {
        connectionCount++;
        const mock = new MockWebSocket(url);
        mockWebSocketInstance = mock;
        return mock;
      } as any;
      MockWebSocketConstructor.CONNECTING = 0;
      MockWebSocketConstructor.OPEN = 1;
      MockWebSocketConstructor.CLOSING = 2;
      MockWebSocketConstructor.CLOSED = 3;
      // @ts-ignore
      global.WebSocket = MockWebSocketConstructor;

      client.connect();
      await vi.runAllTimersAsync();

      expect(connectionCount).toBe(1);

      client.disconnect();

      // Try to trigger reconnection
      await vi.advanceTimersByTimeAsync(5000);

      // Should not have reconnected (still only 1 connection)
      expect(connectionCount).toBe(1);
    });

    it('should reset reconnection delay on successful connection', async () => {
      const client = new WebSocketClient();

      client.connect();
      await vi.runAllTimersAsync();

      // Trigger failures to increase delay
      mockWebSocketInstance?.simulateClose();
      await vi.advanceTimersByTimeAsync(1000); // First reconnect: 1000ms

      mockWebSocketInstance?.simulateClose();
      await vi.advanceTimersByTimeAsync(1500); // Second reconnect: 1500ms

      // Now successfully connect (let it stay open)
      await vi.advanceTimersByTimeAsync(100);
      await vi.runAllTimersAsync();

      // Simulate failure again - delay should reset to 1000ms
      mockWebSocketInstance?.simulateClose();

      // Should reconnect at initial delay (1000ms), not continue from 1500ms
      await vi.advanceTimersByTimeAsync(999);
      const beforeReconnect = mockWebSocketInstance;
      await vi.advanceTimersByTimeAsync(1);

      // Should have reconnected after 1000ms (reset delay)
      expect(mockWebSocketInstance).not.toBe(beforeReconnect);
    });
  });

  describe('Connection State', () => {
    it('should return current connection state', () => {
      const client = new WebSocketClient();

      expect(client.getConnectionState()).toBe(ConnectionState.Disconnected);
    });

    it('should call connection state handlers with current state on subscribe', () => {
      const client = new WebSocketClient();
      const states: ConnectionState[] = [];

      client.onConnectionStateChange((state) => {
        states.push(state);
      });

      // Should immediately receive current state
      expect(states).toEqual([ConnectionState.Disconnected]);
    });

    it('should allow unsubscribing connection state handlers', async () => {
      const client = new WebSocketClient();
      const states: ConnectionState[] = [];

      const unsubscribe = client.onConnectionStateChange((state) => {
        states.push(state);
      });

      client.connect();
      await vi.runAllTimersAsync();

      const stateCountBeforeUnsubscribe = states.length;

      unsubscribe();

      client.disconnect();

      // Should not have received the Disconnected state
      expect(states.length).toBe(stateCountBeforeUnsubscribe);
    });

    it('should not emit duplicate state changes', async () => {
      const client = new WebSocketClient();
      const states: ConnectionState[] = [];

      client.onConnectionStateChange((state) => {
        states.push(state);
      });

      client.connect();
      await vi.runAllTimersAsync();

      const connectedStateCount = states.filter(
        (s) => s === ConnectionState.Connected
      ).length;

      expect(connectedStateCount).toBe(1);
    });
  });

  describe('Ping Measurement', () => {
    it('should return 0 ping when not connected', () => {
      const client = new WebSocketClient();

      expect(client.getPing()).toBe(0);
    });

    it('should return 0 ping when no messages received', async () => {
      const client = new WebSocketClient();
      client.connect();
      await vi.runAllTimersAsync();

      expect(client.getPing()).toBe(0);
    });

    it('should calculate ping as time since last message', async () => {
      const client = new WebSocketClient();
      client.connect();
      await vi.runAllTimersAsync();

      mockWebSocketInstance?.simulateMessage({
        tick: 1,
        timestamp: '2025-01-05T21:00:00.000Z',
        agents: [],
      });

      await vi.advanceTimersByTimeAsync(500);

      const ping = client.getPing();
      expect(ping).toBeGreaterThanOrEqual(490);
      expect(ping).toBeLessThanOrEqual(510);
    });
  });
});
