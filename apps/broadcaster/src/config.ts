import type { AppConfig } from './types.js';

/**
 * Application configuration loaded from environment variables
 */
export const config: AppConfig = {
  nats: {
    // Default to 'nats' hostname (Docker network) or localhost fallback
    servers: process.env.NATS_URL || 'nats://nats:4222',
    subject: 'speciate.agents.transform',
  },
  websocket: {
    port: parseInt(process.env.WS_PORT || '8080', 10),
    path: '/stream',
  },
  logging: {
    level: process.env.LOG_LEVEL || 'info',
  },
};
