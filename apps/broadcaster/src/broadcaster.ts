import type { NatsSubscriber } from './nats-subscriber.js';
import type { WebSocketServer } from './websocket-server.js';
import type { SimulationFrame } from './types.js';
import type { HealthServer } from './health-server.js';

/**
 * Broadcaster coordinates between NATS subscriber and WebSocket server
 * Implements simple pass-through relay for the walking skeleton
 */
export class Broadcaster {
  private lastMessageTime: number = 0;
  private messagesSinceHeartbeat: number = 0;
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private healthServer: HealthServer | null = null;

  constructor(
    private natsSubscriber: NatsSubscriber,
    private wsServer: WebSocketServer
  ) {}

  /**
   * Set the health server instance
   */
  setHealthServer(healthServer: HealthServer): void {
    this.healthServer = healthServer;
  }

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
      this.updateHealthStatus();
    });

    this.natsSubscriber.on('disconnected', () => {
      console.warn('[Broadcaster] NATS disconnected');
      this.updateHealthStatus();
    });

    this.natsSubscriber.on('reconnecting', () => {
      console.log('[Broadcaster] NATS reconnecting...');
      this.updateHealthStatus();
    });

    this.natsSubscriber.on('reconnected', () => {
      console.log('[Broadcaster] NATS reconnected');
      this.updateHealthStatus();
    });

    this.natsSubscriber.on('resubscribed', () => {
      console.log('[Broadcaster] NATS resubscribed successfully');
      this.updateHealthStatus();
    });

    this.natsSubscriber.on('resubscribeFailed', (error: Error) => {
      console.error('[Broadcaster] NATS resubscribe failed:', error.message);
      this.updateHealthStatus();
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
   * Update health server with current status
   */
  private updateHealthStatus(): void {
    if (this.healthServer) {
      this.healthServer.updateNatsStatus(
        this.natsSubscriber.isConnected(),
        this.natsSubscriber.hasActiveSubscription()
      );
      this.healthServer.updateWebSocketStatus(true, this.wsServer.getClientCount());
    }
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

      // Update health status periodically
      this.updateHealthStatus();

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
          `[Broadcaster] Tick ${frame.tick} | ${this.wsServer.getClientCount()} client(s) | ${frame.crits.length} crit(s)`
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
