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
  movementUs: number;
  perceptionUs: number;
  spatialGridRebuildUs: number;
  l1AggregationUs: number;
  behaviorTransitionUs: number;
  steeringUs: number;
  captureDebugAccelUs: number;
  exportPositionsUs: number;
  ipcQueryUs: number;
  ipcSerializeUs: number;
  ipcWriteUs: number;
  ipcFrameDropsTotal: number;
  ipcChannelUtilizationPct: number;
  ipcWriterThreadUs: number;
  // Count metrics (reset-on-read)
  cellsQueriedTotal: number;
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

export enum L1Classification {
  Empty = 0,
  Threat = 1,
  Prey = 2,
  Crowded = 3,
}

export interface L1VisionDebugEntry {
  cellIdx: number;
  classification: L1Classification;
  centerX: number;
  centerY: number;
  directionX: number;
  directionY: number;
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
  l1Vision?: L1VisionDebugEntry[];
}

/** One tick's creature data in wire format: the SoA Float32Array straight off
 *  the NAPI seam ([IDs..., Xs..., Ys..., Rots..., Sizes...] — BufferLayout.ts). */
export interface CreatureSoA {
  buffer: Float32Array;
  count: number;
}

/** Structural view of a decoded SoA frame (the renderer's CreatureFrameSlot
 *  satisfies this). Consumers needing id/position lookups read these typed
 *  arrays instead of materialized objects. */
export interface CreatureFrameView {
  ids: Int32Array;
  xs: Float32Array;
  ys: Float32Array;
  rots: Float32Array;
  sizes: Float32Array;
  count: number;
  idToIndex: ReadonlyMap<number, number>;
}

export interface GameState {
  protocolVersion: number;
  tick: number;
  tickRateHz: number;
  /** Hot-path payload — zero-copy view of the IPC buffer. */
  soa: CreatureSoA;
  /**
   * Materialized object view of `soa`. LAZY: built on first access and cached
   * per state. The render path never touches it — reading it on every tick
   * reintroduces the per-tick AoS fan-out this seam exists to avoid.
   */
  creatures: CreatureData[];
  entityCount?: number;
  systemTimingsUs?: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
  perceptionDebug?: PerceptionDebugData;
}
