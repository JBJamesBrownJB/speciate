import type { MetricStatistics } from '../types';

export function calculateStatistics(values: number[]): MetricStatistics {
  if (values.length === 0) {
    return {
      avg: 0,
      min: 0,
      max: 0,
      stdDev: 0,
      p50: 0,
      p95: 0,
      p99: 0,
    };
  }

  const sorted = [...values].sort((a, b) => a - b);
  const sum = values.reduce((acc, val) => acc + val, 0);
  const avg = sum / values.length;

  const min = sorted[0];
  const max = sorted[sorted.length - 1];

  const variance = values.reduce((acc, val) => acc + Math.pow(val - avg, 2), 0) / values.length;
  const stdDev = Math.sqrt(variance);

  const p50 = percentile(sorted, 0.50);
  const p95 = percentile(sorted, 0.95);
  const p99 = percentile(sorted, 0.99);

  return {
    avg,
    min,
    max,
    stdDev,
    p50,
    p95,
    p99,
  };
}

function percentile(sortedValues: number[], p: number): number {
  const index = (sortedValues.length - 1) * p;
  const lower = Math.floor(index);
  const upper = Math.ceil(index);
  const weight = index - lower;

  if (lower === upper) {
    return sortedValues[lower];
  }

  return sortedValues[lower] * (1 - weight) + sortedValues[upper] * weight;
}
