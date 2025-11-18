/**
 * TypeScript type definitions for dev tools
 */

export interface DevCommand {
  type: 'dev_spawn_creature' | 'dev_load_trial' | 'dev_clear_creatures';
  x?: number;
  y?: number;
  dna?: any;
  template?: string;
}

export interface SystemTimingsSnapshot {
  totalTickUs: number;
  movementUs: number;
  perceptionUs: number;
  behaviorUs: number;
  behaviorTransitionUs: number;
  wanderUs: number;
  fleeUs: number;
  avoidanceUs: number;
  rotationUs: number;
  ipcQueryUs: number;
  ipcSerializeUs: number;
  ipcWriteUs: number;
  ipcFrameDropsTotal: number;
  ipcChannelUtilizationPct: number;
  ipcWriterThreadUs: number;
}

export interface TelemetryFrame {
  tick: number;
  creatureCount: number;
  tickRateHz: number;
  systemTimingsUs: SystemTimingsSnapshot;
}

export interface GameState {
  tick: number;
  creatures: CreatureSnapshot[];
  timestamp_ms: number;
  tickRateHz?: number;
  entityCount?: number;
  systemTimingsUs?: SystemTimingsSnapshot;
}

export interface CreatureSnapshot {
  id: number;
  x: number;
  y: number;
  heading: number;
  body_radius: number;
  energy: number;
}

declare global {
  interface Window {
    electron?: {
      sendCommand?: (command: DevCommand) => void;
      onStateUpdateBinary?: (callback: (binaryData: Uint8Array) => void) => void;
      onTelemetryUpdate?: (callback: (telemetry: TelemetryFrame) => void) => void;
      removeStateUpdateListener?: () => void;
    };
  }
}

export {};
