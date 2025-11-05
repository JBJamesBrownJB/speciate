import { EventEmitter } from 'events';
import { connect, NatsConnection, Subscription } from 'nats';
import { decode } from '@msgpack/msgpack';
import type { NatsConfig, SimulationFrame, AgentTransform } from './types.js';

// Maximum safe integer for JavaScript (2^53 - 1)
const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;

/**
 * Validates that an object is a valid AgentTransform
 */
function isValidAgentTransform(obj: unknown): obj is AgentTransform {
  if (typeof obj !== 'object' || obj === null) {
    return false;
  }

  const agent = obj as Record<string, unknown>;

  // Check required fields exist and have correct types
  if (typeof agent.id !== 'number') return false;
  if (typeof agent.x !== 'number') return false;
  if (typeof agent.y !== 'number') return false;
  if (typeof agent.vx !== 'number') return false;
  if (typeof agent.vy !== 'number') return false;
  if (typeof agent.rotation !== 'number') return false;

  // Check for NaN values
  if (Number.isNaN(agent.x) || Number.isNaN(agent.y)) return false;
  if (Number.isNaN(agent.vx) || Number.isNaN(agent.vy)) return false;
  if (Number.isNaN(agent.rotation)) return false;

  // Check for integer overflow in ID
  if (agent.id > MAX_SAFE_INTEGER) {
    console.warn(`Agent ID ${agent.id} exceeds safe integer range`);
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

  if (!Array.isArray(frame.agents)) {
    console.error('Invalid frame: agents must be an array');
    return false;
  }

  // Check for integer overflow in tick
  if (frame.tick > MAX_SAFE_INTEGER) {
    console.warn(`Tick ${frame.tick} exceeds safe integer range`);
  }

  // Validate each agent
  for (let i = 0; i < frame.agents.length; i++) {
    if (!isValidAgentTransform(frame.agents[i])) {
      console.error(`Invalid agent at index ${i}:`, frame.agents[i]);
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

  constructor(private config: NatsConfig) {
    super();
  }

  /**
   * Connect to NATS server and start subscription
   */
  async connect(): Promise<void> {
    try {
      this.connection = await connect({
        servers: this.config.servers,
      });

      this.emit('connected');

      // Handle connection lifecycle events
      (async () => {
        if (!this.connection) return;

        for await (const status of this.connection.status()) {
          switch (status.type) {
            case 'disconnect':
              this.emit('disconnected');
              break;
            case 'reconnecting':
              this.emit('reconnecting');
              break;
            case 'reconnect':
              this.emit('reconnected');
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
