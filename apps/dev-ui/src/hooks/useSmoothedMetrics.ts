import { useRef } from 'react';
import type { HardwareMetrics } from '../types';

export const useSmoothedMetrics = (
  rawMetrics: HardwareMetrics | undefined,
  alpha: number = 0.3
): HardwareMetrics | undefined => {
  const smoothedRef = useRef<HardwareMetrics | null>(null);

  if (!rawMetrics) return undefined;

  if (!smoothedRef.current) {
    smoothedRef.current = rawMetrics;
    return rawMetrics;
  }

  const smooth = (newVal: number, oldVal: number) =>
    alpha * newVal + (1 - alpha) * oldVal;

  smoothedRef.current = {
    cyclesDelta: Math.round(smooth(rawMetrics.cyclesDelta, smoothedRef.current.cyclesDelta)),
    instructionsDelta: Math.round(smooth(rawMetrics.instructionsDelta, smoothedRef.current.instructionsDelta)),
    cacheRefsDelta: Math.round(smooth(rawMetrics.cacheRefsDelta, smoothedRef.current.cacheRefsDelta)),
    cacheMissesDelta: Math.round(smooth(rawMetrics.cacheMissesDelta, smoothedRef.current.cacheMissesDelta)),
    l1dMissesDelta: Math.round(smooth(rawMetrics.l1dMissesDelta, smoothedRef.current.l1dMissesDelta)),
    l1iMissesDelta: Math.round(smooth(rawMetrics.l1iMissesDelta, smoothedRef.current.l1iMissesDelta)),
    branchInstructionsDelta: Math.round(smooth(rawMetrics.branchInstructionsDelta, smoothedRef.current.branchInstructionsDelta)),
    branchMissesDelta: Math.round(smooth(rawMetrics.branchMissesDelta, smoothedRef.current.branchMissesDelta)),
    stalledFrontendDelta: Math.round(smooth(rawMetrics.stalledFrontendDelta, smoothedRef.current.stalledFrontendDelta)),
    stalledBackendDelta: Math.round(smooth(rawMetrics.stalledBackendDelta, smoothedRef.current.stalledBackendDelta)),
    ipc: smooth(rawMetrics.ipc, smoothedRef.current.ipc),
    l1dMissRate: smooth(rawMetrics.l1dMissRate, smoothedRef.current.l1dMissRate),
    l1iMissRate: smooth(rawMetrics.l1iMissRate, smoothedRef.current.l1iMissRate),
    llcMissRate: smooth(rawMetrics.llcMissRate, smoothedRef.current.llcMissRate),
    branchMissRate: smooth(rawMetrics.branchMissRate, smoothedRef.current.branchMissRate),
    frontendStallRatio: smooth(rawMetrics.frontendStallRatio, smoothedRef.current.frontendStallRatio),
    backendStallRatio: smooth(rawMetrics.backendStallRatio, smoothedRef.current.backendStallRatio),
  };

  return smoothedRef.current;
};
