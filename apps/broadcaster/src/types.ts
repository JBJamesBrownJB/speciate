/**
 * Simulation frame data structure from NATS contract
 */
export interface SimulationFrame {
  tick: number;
  timestamp: string;
  agents: AgentTransform[];
}

/**
 * Agent transform data (position, velocity, rotation)
 */
export interface AgentTransform {
  id: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  rotation: number;
}

/**
 * Configuration for NATS connection
 */
export interface NatsConfig {
  servers: string;
  subject: string;
}

/**
 * Configuration for WebSocket server
 */
export interface WebSocketConfig {
  port: number;
  path: string;
}

/**
 * Logging configuration
 */
export interface LoggingConfig {
  level: string;
}

/**
 * Complete application configuration
 */
export interface AppConfig {
  nats: NatsConfig;
  websocket: WebSocketConfig;
  logging: LoggingConfig;
}
