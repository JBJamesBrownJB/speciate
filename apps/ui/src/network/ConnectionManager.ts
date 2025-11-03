import { NETWORK_CONFIG } from '../core/constants';

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

export interface ConnectionEventHandlers {
  onOpen: () => void;
  onMessage: (data: string) => void;
  onClose: () => void;
  onError: (error: Event) => void;
}

export class ConnectionManager {
  private socket: WebSocket | null = null;
  private state: ConnectionState = 'disconnected';
  private reconnectAttempts = 0;
  private reconnectTimer: number | null = null;

  constructor(
    private url: string,
    private handlers: ConnectionEventHandlers
  ) {}

  connect(): void {
    if (this.state === 'connected' || this.state === 'connecting') {
      return;
    }

    this.state = 'connecting';
    this.socket = new WebSocket(this.url);

    this.socket.onopen = () => this.handleOpen();
    this.socket.onmessage = (event) => this.handleMessage(event);
    this.socket.onclose = () => this.handleClose();
    this.socket.onerror = (error) => this.handleError(error);
  }

  disconnect(): void {
    this.clearReconnectTimer();

    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }

    this.state = 'disconnected';
  }

  send(data: string): void {
    if (this.state === 'connected' && this.socket) {
      this.socket.send(data);
    }
  }

  getState(): ConnectionState {
    return this.state;
  }

  private handleOpen(): void {
    this.state = 'connected';
    this.reconnectAttempts = 0;
    this.handlers.onOpen();
  }

  private handleMessage(event: MessageEvent): void {
    this.handlers.onMessage(event.data);
  }

  private handleClose(): void {
    this.state = 'disconnected';
    this.handlers.onClose();
    this.attemptReconnect();
  }

  private handleError(error: Event): void {
    this.state = 'error';
    this.handlers.onError(error);
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts >= NETWORK_CONFIG.MAX_RECONNECT_ATTEMPTS) {
      console.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    this.reconnectTimer = window.setTimeout(() => {
      console.log(`Reconnection attempt ${this.reconnectAttempts}...`);
      this.connect();
    }, NETWORK_CONFIG.RECONNECT_DELAY_MS);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}
