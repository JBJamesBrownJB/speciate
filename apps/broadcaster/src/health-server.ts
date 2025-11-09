import { createServer, IncomingMessage, ServerResponse } from 'http';
import { logger } from './logger.js';
import type { HealthConfig } from './types.js';

/**
 * Health status for the service
 */
export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  nats: {
    connected: boolean;
    subscribed: boolean;
  };
  websocket: {
    active: boolean;
    clients: number;
  };
  uptime: number;
}

/**
 * Health check HTTP server for monitoring
 */
export class HealthServer {
  private server: ReturnType<typeof createServer> | null = null;
  private startTime: number = Date.now();
  private healthStatus: HealthStatus = {
    status: 'unhealthy',
    nats: {
      connected: false,
      subscribed: false,
    },
    websocket: {
      active: false,
      clients: 0,
    },
    uptime: 0,
  };

  constructor(private config: HealthConfig) {}

  /**
   * Start the health check server
   */
  start(): void {
    this.server = createServer((req: IncomingMessage, res: ServerResponse) => {
      if (req.url === '/health' && req.method === 'GET') {
        this.handleHealthCheck(req, res);
      } else {
        res.writeHead(404, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Not found' }));
      }
    });

    this.server.listen(this.config.port, () => {
      logger.info(`Health check endpoint: http://localhost:${this.config.port}/health`);
    });
  }

  /**
   * Handle health check requests
   */
  private handleHealthCheck(_req: IncomingMessage, res: ServerResponse): void {
    // Update uptime
    this.healthStatus.uptime = Date.now() - this.startTime;

    // Determine overall status
    if (this.healthStatus.nats.connected && this.healthStatus.nats.subscribed) {
      this.healthStatus.status = 'healthy';
    } else if (this.healthStatus.nats.connected && !this.healthStatus.nats.subscribed) {
      this.healthStatus.status = 'degraded';
    } else {
      this.healthStatus.status = 'unhealthy';
    }

    const statusCode = this.healthStatus.status === 'healthy' ? 200 : 503;

    res.writeHead(statusCode, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(this.healthStatus, null, 2));
  }

  /**
   * Update NATS connection status
   */
  updateNatsStatus(connected: boolean, subscribed: boolean): void {
    this.healthStatus.nats.connected = connected;
    this.healthStatus.nats.subscribed = subscribed;
  }

  /**
   * Update WebSocket status
   */
  updateWebSocketStatus(active: boolean, clients: number): void {
    this.healthStatus.websocket.active = active;
    this.healthStatus.websocket.clients = clients;
  }

  /**
   * Stop the health check server
   */
  async stop(): Promise<void> {
    if (this.server) {
      return new Promise((resolve, reject) => {
        this.server!.close((err) => {
          if (err) {
            reject(err);
          } else {
            logger.info('Health check server stopped');
            resolve();
          }
        });
      });
    }
  }
}
