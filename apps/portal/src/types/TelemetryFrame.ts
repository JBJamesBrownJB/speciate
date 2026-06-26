/**
 * Telemetry data structure sent from Rust simulation
 */

export interface HardwareMetrics {
  cyclesDelta: number;
  instructionsDelta: number;
  cacheRefsDelta: number;
  cacheMissesDelta: number;
  l1dMissesDelta: number;
  l1iMissesDelta: number;
  branchInstructionsDelta: number;
  branchMissesDelta: number;
  stalledFrontendDelta: number;
  stalledBackendDelta: number;
  ipc: number;
  l1dMissRate: number;
  l1iMissRate: number;
  llcMissRate: number;
  branchMissRate: number;
  frontendStallRatio: number;
  backendStallRatio: number;
}

export interface ParallelizationMetrics {
  cpuCoresTotal: number;
  cpuCoresActive: number;
  cpuUtilizationPct: number;
  estimatedParallelismFactor: number;
  concurrentSystemsEstimate: number;
}

export interface SystemTimingsSnapshot {
  totalTickUs: number;
  movementUs: number; // Now includes rotation (fused in Sprint 20)
  perceptionUs: number;
  spatialGridRebuildUs: number;
  behaviorTransitionUs: number;
  steeringUs: number; // Fused steering system (Sprint 20)
  captureDebugAccelUs: number;
  archetypeCount: number;
  entityCount: number;
}

export interface TelemetryFrame {
  tick: number;
  creatureCount: number;
  plantCount?: number;
  tickRateHz: number;
  spatialGridCellSize: number;
  l1CellSize: number;
  spatialGridMinX: number;
  spatialGridMaxX: number;
  spatialGridMinY: number;
  spatialGridMaxY: number;
  // Note: L1 cell data now sent via separate binary buffer (onL1BufferUpdate)
  systemTimings: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
  parallelizationMetrics?: ParallelizationMetrics;
  timestamp: number;
  napiBufferCapacityPct?: number;
  napiBufferUsed?: number;
  napiBufferCapacity?: number;
}
