#!/usr/bin/env node

import { config } from './config.js';
import { NatsSubscriber } from './nats-subscriber.js';
import { WebSocketServer } from './websocket-server.js';
import { Broadcaster } from './broadcaster.js';
import { HealthServer } from './health-server.js';
import { setLogLevel, logger } from './logger.js';

/**
 * Exponential backoff retry logic for NATS connection
 */
async function connectWithRetry(
  natsSubscriber: NatsSubscriber,
  maxRetries: number,
  initialDelay: number
): Promise<void> {
  const MAX_DELAY_MS = 30000; // Cap backoff at 30 seconds
  let attempt = 0;

  while (true) {
    try {
      await natsSubscriber.connect();
      logger.info('Successfully connected to NATS');
      return;
    } catch (error) {
      attempt++;

      // Check if we've exceeded max retries (-1 means infinite)
      if (maxRetries !== -1 && attempt >= maxRetries) {
        throw new Error(`Failed to connect to NATS after ${attempt} attempts: ${error}`);
      }

      // Calculate exponential backoff delay (capped at MAX_DELAY_MS)
      const delay = Math.min(initialDelay * Math.pow(2, attempt - 1), MAX_DELAY_MS);

      if (maxRetries === -1) {
        logger.warn(`Failed to connect to NATS (attempt ${attempt}). Retrying in ${delay}ms...`);
      } else {
        logger.warn(
          `Failed to connect to NATS (attempt ${attempt}/${maxRetries}). Retrying in ${delay}ms...`
        );
      }

      // Wait before retrying
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }
}

/**
 * Main entry point for the Broadcaster service
 */
async function main() {
  // Set log level from config
  setLogLevel(config.logging.level as any);

  logger.info('='.repeat(60));
  logger.info('Broadcaster Service Starting');
  logger.info('='.repeat(60));
  logger.info(`NATS Server: ${config.nats.servers}`);
  logger.info(`NATS Subject: ${config.nats.subject}`);
  logger.info(`WebSocket Port: ${config.websocket.port}`);
  logger.info(`WebSocket Path: ${config.websocket.path}`);
  logger.info(`Health Port: ${config.health.port}`);
  logger.info(`Log Level: ${config.logging.level}`);
  logger.info('='.repeat(60));

  // Create service components
  const natsSubscriber = new NatsSubscriber(config.nats);
  const wsServer = new WebSocketServer(config.websocket);
  const healthServer = new HealthServer(config.health);
  const broadcaster = new Broadcaster(natsSubscriber, wsServer);

  // Link health server to broadcaster
  broadcaster.setHealthServer(healthServer);

  // Set up graceful shutdown
  let isShuttingDown = false;

  const shutdown = async (signal: string) => {
    if (isShuttingDown) {
      logger.warn('Shutdown already in progress...');
      return;
    }

    isShuttingDown = true;

    logger.info(`\nReceived ${signal}, shutting down gracefully...`);

    try {
      await broadcaster.shutdown();
      await healthServer.stop();
      logger.info('Broadcaster service stopped successfully');
      process.exit(0);
    } catch (error) {
      logger.error('Error during shutdown:', error);
      process.exit(1);
    }
  };

  // Register signal handlers
  process.on('SIGTERM', () => shutdown('SIGTERM'));
  process.on('SIGINT', () => shutdown('SIGINT'));

  // Handle unhandled promise rejections
  process.on('unhandledRejection', (reason, promise) => {
    logger.error('Unhandled Promise Rejection:', reason);
    logger.error('Promise:', promise);
  });

  // Handle uncaught exceptions
  process.on('uncaughtException', (error) => {
    logger.error('Uncaught Exception:', error);
    shutdown('UNCAUGHT_EXCEPTION').catch(() => process.exit(1));
  });

  try {
    // Start health server first (available even if NATS is down)
    logger.info('Starting health check server...');
    healthServer.start();

    // Connect to NATS with retry logic
    logger.info('Connecting to NATS...');
    await connectWithRetry(natsSubscriber, config.nats.connectMaxRetries, config.nats.connectRetryDelay);

    // Start broadcaster
    logger.info('Starting broadcaster...');
    broadcaster.start();

    logger.info('');
    logger.info('='.repeat(60));
    logger.info('Broadcaster Service Ready');
    logger.info('Listening for simulation data from NATS...');
    logger.info('WebSocket clients can connect to:');
    logger.info(`  ws://localhost:${config.websocket.port}${config.websocket.path}`);
    logger.info('='.repeat(60));
    logger.info('Press Ctrl+C to stop');
    logger.info('');
  } catch (error) {
    logger.error('Failed to start broadcaster service:', error);
    process.exit(1);
  }
}

// Run main function
main().catch((error) => {
  logger.error('Fatal error:', error);
  process.exit(1);
});
