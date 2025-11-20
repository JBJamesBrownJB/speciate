export interface CreatureData {
  id: number;
  x: number;
  y: number;
  rotation: number;
  size: number;
}

export interface HardwareMetrics {
  cycles: number;
  instructions: number;
  cacheReferences: number;
  cacheMisses: number;
  l1Misses: number;
  ipc: number;
  cacheMissRate: number;
  l1MissRate: number;
}

export interface EcsMetrics {
  archetypeCount: number;
  entityCount: number;
  systemTickMs: number;
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
  archetypeCount: number;
  entityCount: number;
}

export interface GameState {
  protocolVersion: number;
  tick: number;
  tickRateHz: number;
  creatures: CreatureData[];
  entityCount?: number;
  systemTimingsUs?: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
}
