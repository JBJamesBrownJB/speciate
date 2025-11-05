#!/usr/bin/env node

import { config } from './config.js';
import { NatsSubscriber } from './nats-subscriber.js';
import { WebSocketServer } from './websocket-server.js';
import { Broadcaster } from './broadcaster.js';
import { setLogLevel, logger } from './logger.js';

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
  logger.info(`Log Level: ${config.logging.level}`);
  logger.info('='.repeat(60));

  // Create service components
  const natsSubscriber = new NatsSubscriber(config.nats);
  const wsServer = new WebSocketServer(config.websocket);
  const broadcaster = new Broadcaster(natsSubscriber, wsServer);

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
    // Connect to NATS
    logger.info('Connecting to NATS...');
    await natsSubscriber.connect();

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
