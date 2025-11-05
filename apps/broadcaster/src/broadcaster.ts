import type { NatsSubscriber } from './nats-subscriber.js';
import type { WebSocketServer } from './websocket-server.js';
import type { SimulationFrame } from './types.js';

/**
 * Broadcaster coordinates between NATS subscriber and WebSocket server
 * Implements simple pass-through relay for the walking skeleton
 */
export class Broadcaster {
  private lastMessageTime: number = 0;
  private messagesSinceHeartbeat: number = 0;
  private heartbeatInterval: NodeJS.Timeout | null = null;

  constructor(
    private natsSubscriber: NatsSubscriber,
    private wsServer: WebSocketServer
  ) {}

  /**
   * Start the broadcaster service
   */
  start(): void {
    console.log('[Broadcaster] Starting service...');

    // Start WebSocket server
    this.wsServer.start();

    // Set up NATS event handlers
    this.setupNatsHandlers();

    // Start heartbeat logging
    this.startHeartbeat();

    console.log('[Broadcaster] Service started');
  }

  /**
   * Set up NATS event handlers
   */
  private setupNatsHandlers(): void {
    // Handle connection events
    this.natsSubscriber.on('connected', () => {
      console.log('[Broadcaster] NATS connected');
    });

    this.natsSubscriber.on('disconnected', () => {
      console.warn('[Broadcaster] NATS disconnected');
    });

    this.natsSubscriber.on('reconnecting', () => {
      console.log('[Broadcaster] NATS reconnecting...');
    });

    this.natsSubscriber.on('reconnected', () => {
      console.log('[Broadcaster] NATS reconnected');
    });

    // Handle messages - simple pass-through relay
    this.natsSubscriber.on('message', (frame: SimulationFrame) => {
      this.handleMessage(frame);
    });

    // Handle errors
    this.natsSubscriber.on('error', (error: Error) => {
      console.error('[Broadcaster] NATS error:', error.message);
    });
  }

  /**
   * Start periodic heartbeat logging
   */
  private startHeartbeat(): void {
    const HEARTBEAT_INTERVAL_MS = 5000; // 5 seconds

    this.heartbeatInterval = setInterval(() => {
      const now = Date.now();
      const timeSinceLastMsg = this.lastMessageTime ? now - this.lastMessageTime : 0;
      const isActive = timeSinceLastMsg < 2000;
      const clientCount = this.wsServer.getClientCount();

      // Only log if there's activity or clients connected
      if (isActive || clientCount > 0) {
        const rate = (this.messagesSinceHeartbeat / (HEARTBEAT_INTERVAL_MS / 1000)).toFixed(1);
        console.log(
          `[Broadcaster] Receiving ${rate} msg/s | ${clientCount} client(s) | ${this.messagesSinceHeartbeat} frames in last 5s`
        );
      }

      this.messagesSinceHeartbeat = 0;
    }, HEARTBEAT_INTERVAL_MS);
  }

  /**
   * Handle incoming simulation frame from NATS
   */
  private handleMessage(frame: SimulationFrame): void {
    // Track message reception for heartbeat
    this.lastMessageTime = Date.now();
    this.messagesSinceHeartbeat++;

    try {
      // Serialize frame to JSON
      const message = JSON.stringify(frame);

      // Broadcast to all WebSocket clients
      this.wsServer.broadcast(message);

      // Optional: Log stats periodically (every 500 ticks, ~25 seconds at 20 Hz)
      if (frame.tick % 500 === 0) {
        console.log(
          `[Broadcaster] Tick ${frame.tick} | ${this.wsServer.getClientCount()} client(s) | ${frame.agents.length} agent(s)`
        );
      }
    } catch (error) {
      console.error('[Broadcaster] Failed to broadcast message:', error);
      // Continue processing - don't crash on single message failure
    }
  }

  /**
   * Shutdown the broadcaster service gracefully
   */
  async shutdown(): Promise<void> {
    console.log('[Broadcaster] Shutting down...');

    // Stop heartbeat
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }

    // Close NATS connection
    await this.natsSubscriber.close();

    // Close WebSocket server
    this.wsServer.close();

    console.log('[Broadcaster] Shutdown complete');
  }
}
