export interface SimulationStateMessage {
  tick: number;
  entity: { x: number; y: number; z: number };
  server_time: number;
}

export function isSimulationStateMessage(data: unknown): data is SimulationStateMessage {
  if (typeof data !== 'object' || data === null) return false;
  const msg = data as Record<string, unknown>;
  return (
    typeof msg.tick === 'number' &&
    typeof msg.server_time === 'number' &&
    typeof msg.entity === 'object' &&
    msg.entity !== null &&
    typeof (msg.entity as Record<string, unknown>).x === 'number' &&
    typeof (msg.entity as Record<string, unknown>).y === 'number' &&
    typeof (msg.entity as Record<string, unknown>).z === 'number'
  );
}
