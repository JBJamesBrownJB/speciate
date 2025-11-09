/**
 * Simulation frame data structure from NATS contract
 */
export interface SimulationFrame {
  tick: number;
  timestamp: string;
  crits: CritTransform[];
}

/**
 * Crit transform data (position, velocity, rotation)
 */
export interface CritTransform {
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
  reconnect: boolean;
  maxReconnectAttempts: number;
  reconnectTimeWait: number;
  timeout: number;
  connectMaxRetries: number;
  connectRetryDelay: number;
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
 * Health check configuration
 */
export interface HealthConfig {
  port: number;
}

/**
 * Complete application configuration
 */
export interface AppConfig {
  nats: NatsConfig;
  websocket: WebSocketConfig;
  logging: LoggingConfig;
  health: HealthConfig;
}
