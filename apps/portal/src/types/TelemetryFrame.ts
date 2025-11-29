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
  movementUs: number;
  perceptionUs: number;
  behaviorUs: number;
  behaviorTransitionUs: number;
  wanderUs: number;
  seekUs: number;
  fleeUs: number;
  avoidanceUs: number;
  rotationUs: number;
  archetypeCount: number;
  entityCount: number;
}

export interface TelemetryFrame {
  tick: number;
  creatureCount: number;
  tickRateHz: number;
  systemTimings: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
  parallelizationMetrics?: ParallelizationMetrics;
  timestamp: number;
  napiBufferCapacityPct?: number;
  napiBufferUsed?: number;
  napiBufferCapacity?: number;
}
