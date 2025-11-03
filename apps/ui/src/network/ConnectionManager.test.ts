import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ConnectionManager, type ConnectionEventHandlers } from './ConnectionManager';

class MockWebSocket {
  onopen: (() => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: ((error: Event) => void) | null = null;

  constructor(public url: string) {}

  send = vi.fn();
  close = vi.fn();
}

describe('ConnectionManager', () => {
  let manager: ConnectionManager;
  let handlers: ConnectionEventHandlers;

  beforeEach(() => {
    global.WebSocket = MockWebSocket as any;

    handlers = {
      onOpen: vi.fn(),
      onMessage: vi.fn(),
      onClose: vi.fn(),
      onError: vi.fn(),
    };

    manager = new ConnectionManager('ws://localhost:8080', handlers);
  });

  it('should start in disconnected state', () => {
    expect(manager.getState()).toBe('disconnected');
  });

  it('should transition to connecting state on connect', () => {
    manager.connect();
    expect(manager.getState()).toBe('connecting');
  });

  it('should call onOpen handler when connection opens', () => {
    manager.connect();
    const socket = (manager as any).socket as MockWebSocket;
    socket.onopen?.();

    expect(handlers.onOpen).toHaveBeenCalled();
    expect(manager.getState()).toBe('connected');
  });

  it('should call onMessage handler when message received', () => {
    manager.connect();
    const socket = (manager as any).socket as MockWebSocket;
    socket.onopen?.();

    const mockEvent = { data: 'test message' } as MessageEvent;
    socket.onmessage?.(mockEvent);

    expect(handlers.onMessage).toHaveBeenCalledWith('test message');
  });

  it('should not connect if already connected', () => {
    manager.connect();
    const socket1 = (manager as any).socket;

    manager.connect();
    const socket2 = (manager as any).socket;

    expect(socket1).toBe(socket2);
  });

  it('should send data when connected', () => {
    manager.connect();
    const socket = (manager as any).socket as MockWebSocket;
    socket.onopen?.();

    manager.send('test data');

    expect(socket.send).toHaveBeenCalledWith('test data');
  });

  it('should not send data when disconnected', () => {
    const socket = new MockWebSocket('ws://test');
    manager.send('test data');

    expect(socket.send).not.toHaveBeenCalled();
  });
});
