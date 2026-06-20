/**
 * TypeScript type definitions for dev tools
 */

export interface DnaData {
  size_gene: number; // 0.0-1.0
  fov_gene: number; // 0.0-1.0
}

export interface DevCommand {
  type: 'dev_spawn_creature' | 'dev_load_trial' | 'dev_clear_creatures' | 'dev_set_system_frequency';
  x?: number;
  y?: number;
  dna?: DnaData;
  template?: string;
  randomizeDna?: boolean;
  systemName?: string;
  divisor?: number;
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

/** Render-pipeline (frontend) metrics — interpolation cadence between the sim and
 *  the renderer. Renderer-origin (portal), relayed via main → dev-ui. Mirrors the
 *  portal's RenderPipelineMetrics. See docs/testing/bugs/jitter-high-populations.md. */
export interface RenderPipelineMetrics {
  distinctGapMeanMs: number;
  distinctGapStdMs: number;
  distinctGapMinMs: number;
  distinctGapMaxMs: number;
  deliveryMeanMs: number;
  alphaResetMean: number;
  alphaResetMin: number;
  alphaResetMax: number;
  stallFrames: number;
  totalFrames: number;
  distinctCount: number;
  duplicateCount: number;
}

/** Windows-only process telemetry (Win32 cycle time + page faults + working set).
 *  `available` is false on non-Windows hosts. Mirrors the Rust WindowsMetricsSnapshot. */
export interface WindowsMetrics {
  available: boolean;
  processCyclesPerSec: number;
  pageFaultsPerSec: number;
  pageFaultCount: number;
  workingSetBytes: number;
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
  // Count metrics (reset-on-read)
  cellsQueriedTotal: number;
  archetypeCount: number;
  entityCount: number;
}

export interface L1CellData {
  x: number;
  y: number;
  totalMass: number;
  creatureCount: number;
}

export interface TelemetryFrame {
  tick: number;
  creatureCount: number;
  tickRateHz: number;
  spatialGridCellSize: number;
  l1CellSize: number;
  l1Cells: L1CellData[];
  systemTimings: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
  parallelizationMetrics?: ParallelizationMetrics;
  windowsMetrics?: WindowsMetrics;
  /** Render-pipeline (frontend lerp) metrics, folded into samples during recording
   *  so the snapshot can capture them (they arrive on a separate live channel). */
  renderMetrics?: RenderPipelineMetrics;
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
  windowsMetrics?: Record<string, MetricStatistics>;
  renderMetrics?: Record<string, MetricStatistics>;
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
      /** Host OS from Node's process.platform (e.g. 'win32', 'linux', 'darwin'). */
      platform?: 'win32' | 'darwin' | 'linux' | string;
      /** DEV-only: render-pipeline metrics relayed from the portal renderer. */
      onRenderMetricsUpdate?: (callback: (metrics: RenderPipelineMetrics) => void) => () => void;
      sendCommand?: (command: DevCommand) => void;
      setSystemFrequency?: (systemName: string, divisor: number) => void;
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
