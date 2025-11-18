export type BehaviorMode = 'Wandering' | 'Fleeing' | 'Feeding' | 'Resting';

export interface Creature {
  id: number;
  x: number;
  y: number;
  rotation: number;
  size: number;
}

export interface SimulationStateMessage {
  tick: number;
  creatures: Creature[];
  server_time: number;
}

export function isSimulationStateMessage(data: unknown): data is SimulationStateMessage {
  if (typeof data !== 'object' || data === null) return false;
  const msg = data as Record<string, unknown>;
  return (
    typeof msg.tick === 'number' &&
    typeof msg.server_time === 'number' &&
    Array.isArray(msg.creatures)
  );
}
