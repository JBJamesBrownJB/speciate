import { vi } from 'vitest';

class MockWebSocket {
  onopen: (() => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: ((error: Event) => void) | null = null;

  constructor(public url: string) {}

  send = vi.fn();
  close = vi.fn();
}

global.WebSocket = MockWebSocket as any;

if (typeof performance === 'undefined') {
  (global as any).performance = {
    now: () => Date.now(),
  };
}
