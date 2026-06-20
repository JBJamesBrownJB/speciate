import { describe, it, expect } from 'vitest';
import { snapshotToTelemetry } from './snapshotConverter';
import type { MetricsSnapshot, MetricStatistics } from '../types';

const stat = (avg: number): MetricStatistics => ({
  avg,
  min: avg,
  max: avg,
  stdDev: 0,
  p50: avg,
  p95: avg,
  p99: avg,
});

const baseSnapshot = (): MetricsSnapshot => ({
  metadata: { sampleCount: 1, durationMs: 1000, startTime: '', endTime: '' },
  tick: stat(10),
  creatureCount: stat(500),
  tickRateHz: stat(20),
  systemTimings: {},
});

describe('snapshotToTelemetry', () => {
  it('rebuilds windowsMetrics from snapshot stats and marks it available', () => {
    const snap: MetricsSnapshot = {
      ...baseSnapshot(),
      windowsMetrics: {
        processCyclesPerSec: stat(2e10),
        pageFaultsPerSec: stat(300),
        pageFaultCount: stat(1000),
        workingSetBytes: stat(1.3e9),
      },
    };
    const frame = snapshotToTelemetry(snap);
    expect(frame.windowsMetrics).toBeDefined();
    // Must be available, otherwise the comparison column hides it.
    expect(frame.windowsMetrics!.available).toBe(true);
    expect(frame.windowsMetrics!.processCyclesPerSec).toBe(2e10);
    expect(frame.windowsMetrics!.workingSetBytes).toBe(1.3e9);
  });

  it('omits windowsMetrics when the snapshot captured none', () => {
    const frame = snapshotToTelemetry(baseSnapshot());
    expect(frame.windowsMetrics).toBeUndefined();
  });
});
