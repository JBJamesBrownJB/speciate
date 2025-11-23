import type { MetricsSnapshot, TelemetryFrame, SystemTimingsSnapshot, HardwareMetrics, ParallelizationMetrics } from '../types';

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

  return {
    tick: Math.round(snapshot.tick.avg),
    creatureCount: Math.round(snapshot.creatureCount.avg),
    tickRateHz: snapshot.tickRateHz.avg,
    systemTimings: systemTimings,
    hardwareMetrics,
    parallelizationMetrics,
    timestamp: Date.now(),
  };
}
