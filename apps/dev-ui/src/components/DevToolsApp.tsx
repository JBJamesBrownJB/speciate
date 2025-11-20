/**
 * Dev Tools Main Application
 *
 * Container for all dev tools functionality:
 * - Spawn creatures manually
 * - Load trial templates
 * - View simulation state
 */

import React, { useState, useEffect, useRef } from 'react';
import { SpawnForm } from './SpawnForm';
import { TrialSelector } from './TrialSelector';
import { StateDisplay } from './StateDisplay';
import { IPCHealthPanel } from './IPCHealthPanel';
import { SystemTimingsPanel } from './SystemTimingsPanel';
import { VectorizationTachometer } from './VectorizationTachometer';
import { CacheFirewall } from './CacheFirewall';
import { BranchScope } from './BranchScope';
import { Parallelism } from './Parallelism';
import { useSmoothedMetrics } from '../hooks/useSmoothedMetrics';
import type { SystemTimingsSnapshot, HardwareMetrics, ParallelizationMetrics, TelemetryFrame } from '../types';
import '../styles/cockpit.css';

export const DevToolsApp: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [tick, setTick] = useState(0);
  const [creatureCount, setCreatureCount] = useState(0);
  const [tickRateHz, setTickRateHz] = useState(0);
  const [systemTimings, setSystemTimings] = useState<SystemTimingsSnapshot | undefined>(undefined);
  const [rawHardwareMetrics, setRawHardwareMetrics] = useState<HardwareMetrics | undefined>(undefined);
  const [parallelizationMetrics, setParallelizationMetrics] = useState<ParallelizationMetrics | undefined>(undefined);
  const hardwareMetrics = useSmoothedMetrics(rawHardwareMetrics, 0.3);
  const lastHardwareUpdateRef = useRef<number>(0);

  useEffect(() => {
    if (window.electron?.sendCommand) {
      setIsConnected(true);
    }

    const handleTelemetryUpdate = (telemetry: TelemetryFrame) => {
      setTick(telemetry.tick);
      setCreatureCount(telemetry.creatureCount);
      setTickRateHz(telemetry.tickRateHz);
      setSystemTimings(telemetry.systemTimingsUs);

      const now = Date.now();
      if (telemetry.hardwareMetrics && (now - lastHardwareUpdateRef.current) >= 200) {
        setRawHardwareMetrics(telemetry.hardwareMetrics);
        lastHardwareUpdateRef.current = now;
      }

      if (telemetry.parallelizationMetrics) {
        setParallelizationMetrics(telemetry.parallelizationMetrics);
      }
    };

    window.electron?.onTelemetryUpdate?.(handleTelemetryUpdate);

    return () => {
      window.electron?.removeStateUpdateListener?.();
    };
  }, []);

  const handleSpawn = (x: number, y: number) => {
    window.electron?.sendCommand?.({
      type: 'dev_spawn_creature',
      x,
      y,
      dna: null,
    });
  };

  const handleLoadTrial = (template: string) => {
    window.electron?.sendCommand?.({
      type: 'dev_load_trial',
      template,
    });
  };

  const handleClearCreatures = () => {
    if (window.confirm('Clear all creatures? This cannot be undone.')) {
      window.electron?.sendCommand?.({
        type: 'dev_clear_creatures',
      });
    }
  };

  return (
    <div>
      <h1>Speciate Dev Tools</h1>

      <div className="status-bar">
        <div className="status-indicator">
          <div className={`status-dot ${isConnected ? '' : 'disconnected'}`} />
          <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
        </div>
        <div>
          Tick: {tick} | Creatures: {creatureCount}
        </div>
      </div>

      <SpawnForm onSpawn={handleSpawn} disabled={!isConnected} />
      <TrialSelector onLoadTrial={handleLoadTrial} disabled={!isConnected} />

      <div className="section">
        <h2>Danger Zone</h2>
        <button
          className="danger"
          onClick={handleClearCreatures}
          disabled={!isConnected}
        >
          Clear All Creatures
        </button>
      </div>

      <StateDisplay tick={tick} creatureCount={creatureCount} tickRateHz={tickRateHz} />

      {hardwareMetrics && (
        <div className="hardware-cockpit-section">
          <h2>Hardware Performance Cockpit</h2>
          <div className="cockpit-container">
            <VectorizationTachometer ipc={hardwareMetrics.ipc} />
            <CacheFirewall
              l1dMissRate={hardwareMetrics.l1dMissRate}
              llcMissRate={hardwareMetrics.llcMissRate}
              backendStallRatio={hardwareMetrics.backendStallRatio}
            />
            <BranchScope branchMissRate={hardwareMetrics.branchMissRate} />
          </div>
          {parallelizationMetrics && systemTimings && (
            <div className="cockpit-container cockpit-row-2">
              <Parallelism metrics={parallelizationMetrics} systemTimings={systemTimings} />
            </div>
          )}
        </div>
      )}

      <IPCHealthPanel timings={systemTimings} />

      <SystemTimingsPanel timings={systemTimings} />
    </div>
  );
};
