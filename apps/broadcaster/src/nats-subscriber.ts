import { EventEmitter } from 'events';
import { connect, NatsConnection, Subscription } from 'nats';
import { decode } from '@msgpack/msgpack';
import type { NatsConfig, SimulationFrame, CritTransform } from './types.js';

// Maximum safe integer for JavaScript (2^53 - 1)
const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;

/**
 * Validates that an object is a valid CritTransform
 */
function isValidCritTransform(obj: unknown): obj is CritTransform {
  if (typeof obj !== 'object' || obj === null) {
    return false;
  }

  const crit = obj as Record<string, unknown>;

  // Check required fields exist and have correct types
  if (typeof crit.id !== 'number') return false;
  if (typeof crit.x !== 'number') return false;
  if (typeof crit.y !== 'number') return false;
  if (typeof crit.vx !== 'number') return false;
  if (typeof crit.vy !== 'number') return false;
  if (typeof crit.rotation !== 'number') return false;

  // Check for NaN values
  if (Number.isNaN(crit.x) || Number.isNaN(crit.y)) return false;
  if (Number.isNaN(crit.vx) || Number.isNaN(crit.vy)) return false;
  if (Number.isNaN(crit.rotation)) return false;

  // Check for integer overflow in ID
  if (crit.id > MAX_SAFE_INTEGER) {
    console.warn(`Crit ID ${crit.id} exceeds safe integer range`);
  }

  return true;
}

/**
 * Validates that an object is a valid SimulationFrame
 */
function isValidSimulationFrame(obj: unknown): obj is SimulationFrame {
  if (typeof obj !== 'object' || obj === null) {
    return false;
  }

  const frame = obj as Record<string, unknown>;

  // Check required fields
  if (typeof frame.tick !== 'number') {
    console.error('Invalid frame: tick must be a number');
    return false;
  }

  if (typeof frame.timestamp !== 'string') {
    console.error('Invalid frame: timestamp must be a string');
    return false;
  }

  if (!Array.isArray(frame.crits)) {
    console.error('Invalid frame: crits must be an array');
    return false;
  }

  // Check for integer overflow in tick
  if (frame.tick > MAX_SAFE_INTEGER) {
    console.warn(`Tick ${frame.tick} exceeds safe integer range`);
  }

  // Validate each crit
  for (let i = 0; i < frame.crits.length; i++) {
    if (!isValidCritTransform(frame.crits[i])) {
      console.error(`Invalid crit at index ${i}:`, frame.crits[i]);
      return false;
    }
  }

  return true;
}

/**
 * Events emitted by NatsSubscriber
 */
export interface NatsSubscriberEvents {
  connected: () => void;
  disconnected: () => void;
  reconnecting: () => void;
  reconnected: () => void;
  resubscribed: () => void;
  resubscribeFailed: (error: Error) => void;
  message: (frame: SimulationFrame) => void;
  error: (error: Error) => void;
}

/**
 * NATS subscriber that connects to NATS server and emits simulation frames
 */
export class NatsSubscriber extends EventEmitter {
  private connection: NatsConnection | null = null;
  private subscription: Subscription | null = null;
  private isClosing = false;
  private subscriptionLoop: Promise<void> | null = null;
  private isSubscribed = false;

  constructor(private config: NatsConfig) {
    super();
  }

  /**
   * Get connection status
   */
  isConnected(): boolean {
    return this.connection !== null && !this.connection.isClosed();
  }

  /**
   * Get subscription status
   */
  hasActiveSubscription(): boolean {
    return this.isSubscribed && this.subscription !== null;
  }

  /**
   * Connect to NATS server and start subscription
   */
  async connect(): Promise<void> {
    try {
      this.connection = await connect({
        servers: this.config.servers,
        reconnect: this.config.reconnect,
        maxReconnectAttempts: this.config.maxReconnectAttempts,
        reconnectTimeWait: this.config.reconnectTimeWait,
        timeout: this.config.timeout,
      });

      this.emit('connected');

      // Handle connection lifecycle events
      (async () => {
        if (!this.connection) return;

        for await (const status of this.connection.status()) {
          switch (status.type) {
            case 'disconnect':
              this.isSubscribed = false;
              this.emit('disconnected');
              break;
            case 'reconnecting':
              this.emit('reconnecting');
              break;
            case 'reconnect':
              this.emit('reconnected');
              // Automatically resubscribe after reconnection
              await this.resubscribe();
              break;
          }
        }
      })().catch((err) => {
        if (!this.isClosing) {
          this.emit('error', err);
        }
      });

      // Start subscription
      await this.subscribe();
    } catch (error) {
      this.emit('error', error as Error);
      throw error;
    }
  }

  /**
   * Subscribe to NATS subject and emit messages
   */
  private async subscribe(): Promise<void> {
    if (!this.connection) {
      throw new Error('Not connected to NATS');
    }

    this.subscription = this.connection.subscribe(this.config.subject);
    this.isSubscribed = true;

    // Process messages in the background
    this.subscriptionLoop = (async () => {
      if (!this.subscription) return;

      try {
        for await (const msg of this.subscription) {
          try {
            // Decode message data as MessagePack
            const decoded = decode(msg.data);

            // Validate the decoded data structure
            if (!isValidSimulationFrame(decoded)) {
              this.emit('error', new Error('Invalid SimulationFrame structure'));
              continue; // Skip this message, continue processing
            }

            // Type assertion is now safe after validation
            const frame = decoded as SimulationFrame;

            // Emit the validated frame
            this.emit('message', frame);
          } catch (decodeError) {
            // Emit error but continue processing
            this.emit('error', new Error(`Failed to decode msgpack message: ${decodeError}`));
          }
        }
      } catch (error) {
        if (!this.isClosing) {
          this.emit('error', error as Error);
        }
      }
    })();
  }

  /**
   * Resubscribe to NATS subject after reconnection
   */
  private async resubscribe(): Promise<void> {
    try {
      if (!this.connection) {
        throw new Error('Not connected to NATS');
      }

      // Clean up old subscription if it exists
      if (this.subscription) {
        this.subscription.unsubscribe();
        this.subscription = null;
      }

      // Wait for previous subscription loop to finish
      if (this.subscriptionLoop) {
        await this.subscriptionLoop.catch(() => {
          // Ignore errors from old subscription
        });
        this.subscriptionLoop = null;
      }

      // Create new subscription
      await this.subscribe();
      this.emit('resubscribed');
    } catch (error) {
      this.isSubscribed = false;
      this.emit('resubscribeFailed', error as Error);
      throw error;
    }
  }

  /**
   * Close the NATS connection and clean up
   */
  async close(): Promise<void> {
    this.isClosing = true;

    if (this.subscription) {
      this.subscription.unsubscribe();
      this.subscription = null;
    }

    if (this.subscriptionLoop) {
      await this.subscriptionLoop.catch(() => {
        // Ignore errors during cleanup
      });
      this.subscriptionLoop = null;
    }

    if (this.connection) {
      await this.connection.close();
      this.connection = null;
    }

    this.removeAllListeners();
  }

  /**
   * Type-safe event emitter methods
   */
  override on<K extends keyof NatsSubscriberEvents>(
    event: K,
    listener: NatsSubscriberEvents[K]
  ): this {
    return super.on(event, listener);
  }

  override emit<K extends keyof NatsSubscriberEvents>(
    event: K,
    ...args: Parameters<NatsSubscriberEvents[K]>
  ): boolean {
    return super.emit(event, ...args);
  }
}
