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

export interface GameState {
  tick: number;
  creatures: CreatureSnapshot[];
  timestamp_ms: number;
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
