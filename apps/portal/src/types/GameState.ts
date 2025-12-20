export interface CreatureData {
  id: number;
  x: number;
  y: number;
  rotation: number;
  size: number;
  ax?: number;
  ay?: number;
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
  movementUs: number; // Now includes rotation (fused in Sprint 20)
  perceptionUs: number;
  spatialGridRebuildUs: number;
  behaviorTransitionUs: number;
  steeringUs: number; // Fused steering system (Sprint 20)
  captureDebugAccelUs: number;
  exportPositionsUs: number; // IPC buffer export with parallel sort (Sprint 16)
  ipcQueryUs: number;
  ipcSerializeUs: number;
  ipcWriteUs: number;
  ipcFrameDropsTotal: number;
  ipcChannelUtilizationPct: number;
  ipcWriterThreadUs: number;
  archetypeCount: number;
  entityCount: number;
}

export interface NeighborDebugInfo {
  id: number;
  x: number;
  y: number;
}

export interface QueriedCell {
  x: number;
  y: number;
}

export interface PerceptionDebugData {
  entityId: number;
  x: number;
  y: number;
  perceptionRange: number;
  queryRadius: number;
  fovAngle: number;
  rotation: number;
  ax: number;
  ay: number;
  neighbors: NeighborDebugInfo[];
  cellSize: number;
  creatureCell: QueriedCell;
  queriedCells: QueriedCell[];
  checkedCells: QueriedCell[];
}

export interface GameState {
  protocolVersion: number;
  tick: number;
  tickRateHz: number;
  creatures: CreatureData[];
  entityCount?: number;
  systemTimingsUs?: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
  perceptionDebug?: PerceptionDebugData;
}
