import type { SimulationStateMessage } from '@/types/messages';
import { isSimulationStateMessage } from '@/types/messages';
import { ConnectionState } from '@/types/entities';

export type MessageHandler = (message: SimulationStateMessage) => void;
export type ConnectionStateHandler = (state: ConnectionState) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectDelay: number;
  private reconnectTimeout: number | null = null;
  private messageHandlers: Set<MessageHandler> = new Set();
  private connectionStateHandlers: Set<ConnectionStateHandler> = new Set();
  private currentState: ConnectionState = ConnectionState.Disconnected;
  private intentionalClose: boolean = false;
  private lastMessageTime: number = 0;
  private readonly maxReconnectDelay = 30000;
  private readonly initialReconnectDelay = 1000;
  private readonly reconnectBackoffMultiplier = 1.5;

  constructor(url: string = 'ws://localhost:8080/ws') {
    this.url = url;
    this.reconnectDelay = this.initialReconnectDelay;
  }

  public connect(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) return;
    this.intentionalClose = false;
    this.setConnectionState(ConnectionState.Connecting);
    try {
      this.ws = new WebSocket(this.url);
      this.setupEventHandlers();
    } catch (error) {
      console.error('[WebSocket] Connection error:', error);
      this.scheduleReconnect();
    }
  }

  public disconnect(): void {
    this.intentionalClose = true;
    this.clearReconnectTimeout();
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
    this.setConnectionState(ConnectionState.Disconnected);
  }

  public onMessage(handler: MessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => this.messageHandlers.delete(handler);
  }

  public onConnectionStateChange(handler: ConnectionStateHandler): () => void {
    this.connectionStateHandlers.add(handler);
    handler(this.currentState);
    return () => this.connectionStateHandlers.delete(handler);
  }

  public getConnectionState(): ConnectionState {
    return this.currentState;
  }

  public getPing(): number {
    if (this.lastMessageTime === 0 || this.currentState !== ConnectionState.Connected) return 0;
    return Date.now() - this.lastMessageTime;
  }

  private setupEventHandlers(): void {
    if (!this.ws) return;
    this.ws.onopen = () => {
      this.reconnectDelay = this.initialReconnectDelay;
      this.setConnectionState(ConnectionState.Connected);
    };
    this.ws.onmessage = (event: MessageEvent) => {
      this.lastMessageTime = Date.now();
      try {
        const parsed: unknown = JSON.parse(event.data);
        if (isSimulationStateMessage(parsed)) {
          this.messageHandlers.forEach(handler => handler(parsed));
        }
      } catch (error) {
        console.error('[WebSocket] Parse error:', error);
      }
    };
    this.ws.onerror = () => {};
    this.ws.onclose = () => {
      this.ws = null;
      if (!this.intentionalClose) {
        this.setConnectionState(ConnectionState.Reconnecting);
        this.scheduleReconnect();
      } else {
        this.setConnectionState(ConnectionState.Disconnected);
      }
    };
  }

  private scheduleReconnect(): void {
    this.clearReconnectTimeout();
    this.reconnectTimeout = window.setTimeout(() => {
      this.connect();
      this.reconnectDelay = Math.min(
        this.reconnectDelay * this.reconnectBackoffMultiplier,
        this.maxReconnectDelay
      );
    }, this.reconnectDelay);
  }

  private clearReconnectTimeout(): void {
    if (this.reconnectTimeout !== null) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
  }

  private setConnectionState(state: ConnectionState): void {
    if (this.currentState === state) return;
    this.currentState = state;
    this.connectionStateHandlers.forEach(handler => handler(state));
  }
}
