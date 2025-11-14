/**
 * Game State Types
 * Shared between Rust simulation and Electron IPC
 */

export interface CreatureData {
  id: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  rotation: number;
  width: number;
  height: number;
  behavior: string;
  energy?: number;
  age: number;
}

export interface GameState {
  tick: number;
  tickRateHz: number; // Measured tick rate (not target)
  creatures: CreatureData[];
}
