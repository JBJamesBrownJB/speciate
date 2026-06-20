import type { MetricsSnapshot, TelemetryFrame, SystemTimingsSnapshot, HardwareMetrics, ParallelizationMetrics, WindowsMetrics } from '../types';

export function snapshotToTelemetry(snapshot: MetricsSnapshot): TelemetryFrame {
  const systemTimings: SystemTimingsSnapshot = {} as SystemTimingsSnapshot;
  for (const [key, stats] of Object.entries(snapshot.systemTimings)) {
    (systemTimings as any)[key] = stats.avg;
  }

  let hardwareMetrics: HardwareMetrics | undefined;
  if (snapshot.hardwareMetrics && snapshot.hardwareMetricsDerived) {
    hardwareMetrics = {} as HardwareMetrics;

    for (const [key, stats] of Object.entries(snapshot.hardwareMetrics)) {
      (hardwareMetrics as any)[key] = stats.avg;
    }

    hardwareMetrics.ipc = snapshot.hardwareMetricsDerived.ipc;
    hardwareMetrics.l1dMissRate = snapshot.hardwareMetricsDerived.l1dMissRate;
    hardwareMetrics.l1iMissRate = snapshot.hardwareMetricsDerived.l1iMissRate;
    hardwareMetrics.llcMissRate = snapshot.hardwareMetricsDerived.llcMissRate;
    hardwareMetrics.branchMissRate = snapshot.hardwareMetricsDerived.branchMissRate;
    hardwareMetrics.frontendStallRatio = snapshot.hardwareMetricsDerived.frontendStallRatio;
    hardwareMetrics.backendStallRatio = snapshot.hardwareMetricsDerived.backendStallRatio;
  } else if (snapshot.hardwareMetrics) {
    hardwareMetrics = {} as HardwareMetrics;
    for (const [key, stats] of Object.entries(snapshot.hardwareMetrics)) {
      (hardwareMetrics as any)[key] = stats.avg;
    }
  }

  let parallelizationMetrics: ParallelizationMetrics | undefined;
  if (snapshot.parallelizationMetrics) {
    parallelizationMetrics = {} as ParallelizationMetrics;
    for (const [key, stats] of Object.entries(snapshot.parallelizationMetrics)) {
      (parallelizationMetrics as any)[key] = stats.avg;
    }
  }

  let windowsMetrics: WindowsMetrics | undefined;
  if (snapshot.windowsMetrics) {
    windowsMetrics = { available: true } as WindowsMetrics;
    for (const [key, stats] of Object.entries(snapshot.windowsMetrics)) {
      (windowsMetrics as any)[key] = stats.avg;
    }
  }

  return {
    tick: Math.round(snapshot.tick.avg),
    creatureCount: Math.round(snapshot.creatureCount.avg),
    tickRateHz: snapshot.tickRateHz.avg,
    spatialGridCellSize: 50, // Default cell size (from Rust CELL_SIZE constant)
    l1CellSize: 60, // L1_CELL_SIZE = CELL_SIZE * 3 (not captured in stat snapshots)
    l1Cells: [], // heatmap cells aren't part of a stat snapshot
    systemTimings: systemTimings,
    hardwareMetrics,
    parallelizationMetrics,
    windowsMetrics,
    timestamp: Date.now(),
  };
}
