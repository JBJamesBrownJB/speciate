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
      onStateUpdate?: (callback: (state: GameState) => void) => void;
      removeStateUpdateListener?: () => void;
    };
  }
}

export {};
