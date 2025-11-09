import type { AppConfig } from './types.js';

/**
 * Application configuration loaded from environment variables
 */
export const config: AppConfig = {
  nats: {
    // Default to 'nats' hostname (Docker network) or localhost fallback
    servers: process.env.NATS_URL || 'nats://nats:4222',
    subject: 'speciate.crits.transform',

    // NATS client reconnection options
    reconnect: process.env.NATS_RECONNECT !== 'false', // Default: true
    maxReconnectAttempts: parseInt(process.env.NATS_MAX_RECONNECT_ATTEMPTS || '-1', 10), // -1 = infinite
    reconnectTimeWait: parseInt(process.env.NATS_RECONNECT_TIME_WAIT_MS || '2000', 10),
    timeout: parseInt(process.env.NATS_TIMEOUT_MS || '20000', 10),

    // Initial connection retry options (at startup)
    connectMaxRetries: parseInt(process.env.NATS_CONNECT_MAX_RETRIES || '-1', 10), // -1 = infinite
    connectRetryDelay: parseInt(process.env.NATS_CONNECT_RETRY_DELAY_MS || '1000', 10),
  },
  websocket: {
    port: parseInt(process.env.WS_PORT || '8080', 10),
    path: '/stream',
  },
  logging: {
    level: process.env.LOG_LEVEL || 'info',
  },
  health: {
    port: parseInt(process.env.HEALTH_PORT || '3001', 10),
  },
};
