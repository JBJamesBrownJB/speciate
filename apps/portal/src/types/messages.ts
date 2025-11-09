export type BehaviorMode = 'Wandering' | 'Fleeing' | 'Feeding' | 'Resting';

export interface Creature {
  id: number;
  x: number;
  y: number;
  rotation: number;
  width: number;
  height: number;
  behavior?: BehaviorMode;
  energy?: number;
  species_id?: number;
}

export interface SimulationStateMessage {
  tick: number;
  creatures: Creature[];
  server_time: number;
}

// New types for broadcaster format
export interface CritTransform {
  id: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  rotation: number;
}

export interface SimulationFrame {
  tick: number;
  timestamp: string;
  crits: CritTransform[];
}

export interface WorldStateMessage {
  type: 'WorldState';
  entities: Array<{
    id: string;
    position: { x: number; y: number };
    orientation: number;
    radius: number;
  }>;
}

export interface EntityUpdateMessage {
  type: 'EntityUpdate';
  entity_id: string;
  position: { x: number; y: number };
  orientation: number;
}

export type ServerMessage = WorldStateMessage | EntityUpdateMessage | { type: string; [key: string]: any };

export function isSimulationStateMessage(data: unknown): data is SimulationStateMessage {
  if (typeof data !== 'object' || data === null) return false;
  const msg = data as Record<string, unknown>;
  return (
    typeof msg.tick === 'number' &&
    typeof msg.server_time === 'number' &&
    Array.isArray(msg.creatures)
  );
}

export function isSimulationFrame(data: unknown): data is SimulationFrame {
  if (typeof data !== 'object' || data === null) return false;
  const msg = data as Record<string, unknown>;
  return (
    typeof msg.tick === 'number' &&
    typeof msg.timestamp === 'string' &&
    Array.isArray(msg.crits)
  );
}

/**
 * Adapts SimulationFrame (broadcaster format) to SimulationStateMessage (Portal format)
 */
export function adaptSimulationFrame(frame: SimulationFrame): SimulationStateMessage {
  const creatures: Creature[] = frame.crits.map((crit) => ({
    id: crit.id,
    x: crit.x,
    y: crit.y,
    rotation: crit.rotation,
    width: 1, // Default width: 1 meter
    height: 1, // Default height: 1 meter
    // Optional fields can be undefined
  }));

  return {
    tick: frame.tick,
    creatures,
    server_time: new Date(frame.timestamp).getTime(),
  };
}
