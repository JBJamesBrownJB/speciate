import { WebSocketServer as WSServer, WebSocket } from 'ws';
import type { WebSocketConfig } from './types.js';

/**
 * WebSocket server that manages client connections and broadcasts messages
 */
export class WebSocketServer {
  private wss: WSServer | null = null;
  private clients = new Set<WebSocket>();

  constructor(private config: WebSocketConfig) {}

  /**
   * Start the WebSocket server
   */
  start(): void {
    this.wss = new WSServer({
      port: this.config.port,
      path: this.config.path,
      // Enable per-message deflate compression (reduces bandwidth by ~30-50%)
      perMessageDeflate: {
        zlibDeflateOptions: {
          level: 6, // Balance between speed and compression (0-9, 6 is default)
        },
        threshold: 1024, // Only compress messages larger than 1KB
      },
    });

    this.wss.on('connection', (ws: WebSocket) => {
      this.handleConnection(ws);
    });

    this.wss.on('error', (error: Error) => {
      console.error('[WebSocketServer] Server error:', error);
    });

    console.log(`[WebSocketServer] Listening on port ${this.config.port}, path ${this.config.path}`);
  }

  /**
   * Handle new client connection
   */
  private handleConnection(ws: WebSocket): void {
    // Add client to set
    this.clients.add(ws);
    console.log(`[WebSocketServer] Client connected (total: ${this.clients.size})`);

    // Handle client disconnection
    ws.on('close', () => {
      this.clients.delete(ws);
      console.log(`[WebSocketServer] Client disconnected (total: ${this.clients.size})`);
    });

    // Handle client errors
    ws.on('error', (error: Error) => {
      console.error('[WebSocketServer] Client error:', error.message);
      // Remove client on error
      this.clients.delete(ws);
      ws.close();
    });
  }

  /**
   * Broadcast message to all connected clients
   */
  broadcast(message: string): void {
    let successCount = 0;
    let errorCount = 0;

    this.clients.forEach((client) => {
      // Only send to clients in OPEN state
      if (client.readyState === WebSocket.OPEN) {
        try {
          client.send(message);
          successCount++;
        } catch (error) {
          errorCount++;
          console.error('[WebSocketServer] Failed to send to client:', error);
          // Clean up failed client
          this.clients.delete(client);
          client.close();
        }
      }
    });

    // Log broadcast stats only if there were errors or for debugging
    if (errorCount > 0) {
      console.log(`[WebSocketServer] Broadcast: ${successCount} success, ${errorCount} errors`);
    }
  }

  /**
   * Get the number of connected clients
   */
  getClientCount(): number {
    return this.clients.size;
  }

  /**
   * Close the WebSocket server and all connections
   */
  close(): void {
    // Close all client connections
    this.clients.forEach((client) => {
      client.close();
    });
    this.clients.clear();

    // Close the server
    if (this.wss) {
      this.wss.close();
      this.wss = null;
    }

    console.log('[WebSocketServer] Server closed');
  }
}
