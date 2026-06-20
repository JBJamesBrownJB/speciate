/**
 * MetricsColumn Component
 *
 * Displays a complete column of performance metrics panels.
 * Used in both single-column (normal) and dual-column (comparison) modes.
 *
 * TODO: Add React Testing Library tests when test framework is configured:
 * - Verify all panels render in correct order
 * - Test prop passing to child components
 * - Test memoization prevents unnecessary re-renders
 */

import React from 'react';
import { SystemTimingsPanel } from './SystemTimingsPanel';
import { VectorizationTachometer } from './VectorizationTachometer';
import { CacheFirewall } from './CacheFirewall';
import { BranchScope } from './BranchScope';
import { Parallelism } from './Parallelism';
import { MemoryMetrics } from './MemoryMetrics';
import { LinuxOnlyBadge } from './LinuxOnlyBadge';
import { hardwareCountersSupported } from '../utils/platform';
import type { SystemTimingsSnapshot, HardwareMetrics, ParallelizationMetrics } from '../types';

export interface MetricsColumnProps {
  label: string;
  labelClass: string;
  tick: number;
  creatureCount: number;
  tickRateHz: number;
  systemTimings?: SystemTimingsSnapshot;
  hardwareMetrics?: HardwareMetrics;
  parallelizationMetrics?: ParallelizationMetrics;
}

/**
 * MetricsColumn - Pure presentational component for rendering metrics
 *
 * Follows SOLID principles:
 * - Single Responsibility: Only renders metrics, no business logic
 * - Open/Closed: Can add new panels without modifying existing code
 * - Dependency Inversion: Depends on props interface (abstraction)
 */
export const MetricsColumn: React.FC<MetricsColumnProps> = React.memo(
  ({
    label,
    labelClass,
    systemTimings,
    hardwareMetrics,
    parallelizationMetrics,
  }) => {
    return (
      <div className="comparison-column">
        <h2 className={`comparison-header ${labelClass}`}>{label}</h2>

        <div className="hardware-cockpit-section">
          <h2>Hardware Performance Cockpit</h2>

          {/* Linux-only PMU gauges, or a "Linux only" badge on non-Linux hosts. */}
          {hardwareMetrics ? (
            <div className="cockpit-container">
              <VectorizationTachometer ipc={hardwareMetrics.ipc} />
              <CacheFirewall
                l1dMissRate={hardwareMetrics.l1dMissRate}
                llcMissRate={hardwareMetrics.llcMissRate}
              />
              <BranchScope branchMissRate={hardwareMetrics.branchMissRate} />
            </div>
          ) : !hardwareCountersSupported() ? (
            <LinuxOnlyBadge />
          ) : null}

          {/* Parallelism + memory are cross-platform — render independently of the
              Linux-only hardware counters so they still appear on Windows. */}
          {parallelizationMetrics && systemTimings && (
            <>
              <div className="cockpit-container cockpit-row-2">
                <Parallelism metrics={parallelizationMetrics} systemTimings={systemTimings} />
              </div>
              <div className="cockpit-container cockpit-row-3">
                <MemoryMetrics processMemoryBytes={parallelizationMetrics.processMemoryBytes} />
              </div>
            </>
          )}
        </div>

        <SystemTimingsPanel timings={systemTimings} />
      </div>
    );
  },
  // Custom equality check: only re-render if props actually changed
  (prevProps, nextProps) => {
    return (
      prevProps.label === nextProps.label &&
      prevProps.tick === nextProps.tick &&
      prevProps.creatureCount === nextProps.creatureCount &&
      prevProps.tickRateHz === nextProps.tickRateHz &&
      prevProps.systemTimings === nextProps.systemTimings &&
      prevProps.hardwareMetrics === nextProps.hardwareMetrics &&
      prevProps.parallelizationMetrics === nextProps.parallelizationMetrics
    );
  }
);

MetricsColumn.displayName = 'MetricsColumn';
