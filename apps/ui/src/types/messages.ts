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
