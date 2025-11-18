export interface CreatureData {
  id: number;
  x: number;
  y: number;
  rotation: number;
  size: number;
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
}

export interface GameState {
  protocolVersion: number;
  tick: number;
  tickRateHz: number;
  creatures: CreatureData[];
  entityCount?: number;
  systemTimingsUs?: SystemTimingsSnapshot;
}
