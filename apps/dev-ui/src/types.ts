/**
 * TypeScript type definitions for dev tools
 */

export interface DnaData {
  size_gene: number; // 0.0-1.0
  fov_gene: number; // 0.0-1.0
}

export interface DevCommand {
  type: 'dev_spawn_creature' | 'dev_load_trial' | 'dev_clear_creatures';
  x?: number;
  y?: number;
  dna?: DnaData;
  template?: string;
  randomizeDna?: boolean;
}

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
  processMemoryBytes: number;
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
  archetypeCount: number;
  entityCount: number;
}

export interface TelemetryFrame {
  tick: number;
  creatureCount: number;
  tickRateHz: number;
  spatialGridCellSize: number;
  systemTimings: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
  parallelizationMetrics?: ParallelizationMetrics;
  timestamp: number;
  napiBufferCapacityPct?: number;
  napiBufferUsed?: number;
  napiBufferCapacity?: number;
}

export interface MetricStatistics {
  avg: number;
  min: number;
  max: number;
  stdDev: number;
  p50: number;
  p95: number;
  p99: number;
}

export interface HardwareMetricsDerived {
  ipc: number;
  l1dMissRate: number;
  l1iMissRate: number;
  llcMissRate: number;
  branchMissRate: number;
  frontendStallRatio: number;
  backendStallRatio: number;
}

export interface MetricsSnapshot {
  metadata: {
    sampleCount: number;
    durationMs: number;
    startTime: string;
    endTime: string;
  };
  tick: MetricStatistics;
  creatureCount: MetricStatistics;
  tickRateHz: MetricStatistics;
  systemTimings: Record<string, MetricStatistics>;
  hardwareMetrics?: Record<string, MetricStatistics>;
  hardwareMetricsDerived?: HardwareMetricsDerived;
  parallelizationMetrics?: Record<string, MetricStatistics>;
}

export interface GameState {
  tick: number;
  creatures: CreatureSnapshot[];
  timestamp_ms: number;
  tickRateHz?: number;
  entityCount?: number;
  systemTimingsUs?: SystemTimingsSnapshot;
}

export interface CreatureSnapshot {
  id: number;
  x: number;
  y: number;
  heading: number;
  body_radius: number;
  energy: number;
}

declare global {
  interface Window {
    electron?: {
      sendCommand?: (command: DevCommand) => void;
      onStateUpdateBinary?: (callback: (binaryData: Uint8Array) => void) => void;
      onTelemetryUpdate?: (callback: (telemetry: TelemetryFrame) => void) => void;
      removeStateUpdateListener?: () => void;
      saveMetricsSnapshot?: (snapshot: MetricsSnapshot) => Promise<{ success: boolean; path?: string; error?: string }>;
      loadMetricsSnapshot?: () => Promise<MetricsSnapshot | null>;
      resizeWindow?: (width: number) => Promise<{ success: boolean; error?: string }>;
    };
  }
}

export {};
