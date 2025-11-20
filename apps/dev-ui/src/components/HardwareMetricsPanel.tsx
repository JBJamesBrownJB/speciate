import React from 'react';
import type { HardwareMetrics } from '../types';

interface Props {
  metrics?: HardwareMetrics;
}

const formatIPC = (ipc: number): string => ipc.toFixed(2);
const formatPercent = (value: number): string => `${value.toFixed(1)}%`;
const formatCount = (value: number): string => value.toLocaleString();

const getIPCClass = (ipc: number): string => {
  if (ipc > 1.5) return 'excellent';
  if (ipc >= 0.5) return 'acceptable';
  return 'poor';
};

const getIPCLabel = (ipc: number): string => {
  if (ipc > 1.5) return '(Vector/SIMD)';
  if (ipc >= 0.5) return '(Scalar)';
  return '(Stall)';
};

const getCacheMissClass = (rate: number): string => {
  if (rate < 5) return 'excellent';
  if (rate < 10) return 'acceptable';
  return 'poor';
};

const getBranchMissClass = (rate: number): string => {
  if (rate < 5) return 'excellent';
  if (rate < 10) return 'acceptable';
  return 'poor';
};

const getStallClass = (ratio: number): string => {
  if (ratio < 10) return 'excellent';
  if (ratio < 20) return 'acceptable';
  return 'poor';
};

export const HardwareMetricsPanel: React.FC<Props> = ({ metrics }) => {
  if (!metrics) {
    return (
      <div className="panel">
        <div className="panel-header">Hardware Metrics (Per-Tick)</div>
        <div className="panel-content">
          <p className="no-data">No hardware metrics available</p>
          <p className="hint">Run simulation with --features dev-tools</p>
        </div>
      </div>
    );
  }

  return (
    <div className="panel">
      <div className="panel-header">Hardware Metrics (Per-Tick Deltas)</div>
      <div className="panel-content">
        <div className="metric-grid">
          <div className="metric-row">
            <span className="metric-label">IPC:</span>
            <span className={`metric-value ${getIPCClass(metrics.ipc)}`}>
              {formatIPC(metrics.ipc)}
            </span>
            <span className="metric-unit">{getIPCLabel(metrics.ipc)}</span>
          </div>

          <div className="metric-row">
            <span className="metric-label">L1D Miss:</span>
            <span className={`metric-value ${getCacheMissClass(metrics.l1dMissRate)}`}>
              {formatPercent(metrics.l1dMissRate)}
            </span>
          </div>

          <div className="metric-row">
            <span className="metric-label">L1I Miss:</span>
            <span className={`metric-value ${getCacheMissClass(metrics.l1iMissRate)}`}>
              {formatPercent(metrics.l1iMissRate)}
            </span>
          </div>

          <div className="metric-row">
            <span className="metric-label">LLC Miss:</span>
            <span className={`metric-value ${getCacheMissClass(metrics.llcMissRate)}`}>
              {formatPercent(metrics.llcMissRate)}
            </span>
          </div>

          <div className="metric-row">
            <span className="metric-label">Branch Miss:</span>
            <span className={`metric-value ${getBranchMissClass(metrics.branchMissRate)}`}>
              {formatPercent(metrics.branchMissRate)}
            </span>
          </div>

          <div className="metric-row">
            <span className="metric-label">Frontend Stall:</span>
            <span className={`metric-value ${getStallClass(metrics.frontendStallRatio)}`}>
              {formatPercent(metrics.frontendStallRatio)}
            </span>
          </div>

          <div className="metric-row">
            <span className="metric-label">Backend Stall:</span>
            <span className={`metric-value ${getStallClass(metrics.backendStallRatio)}`}>
              {formatPercent(metrics.backendStallRatio)}
            </span>
          </div>

          <div className="metric-row metric-row-secondary">
            <span className="metric-label">Δ Cycles:</span>
            <span className="metric-value-small">
              {formatCount(metrics.cyclesDelta)}
            </span>
          </div>

          <div className="metric-row metric-row-secondary">
            <span className="metric-label">Δ Instructions:</span>
            <span className="metric-value-small">
              {formatCount(metrics.instructionsDelta)}
            </span>
          </div>

          <div className="metric-row metric-row-secondary">
            <span className="metric-label">Δ Branches:</span>
            <span className="metric-value-small">
              {formatCount(metrics.branchInstructionsDelta)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};
