/**
 * Dev Tools Main Application
 *
 * Container for all dev tools functionality:
 * - Spawn creatures manually
 * - Load trial templates
 * - View simulation state
 */

import React, { useState, useEffect } from 'react';
import { SpawnForm } from './SpawnForm';
import { TrialSelector } from './TrialSelector';
import { StateDisplay } from './StateDisplay';
import { IPCHealthPanel } from './IPCHealthPanel';
import { SystemTimingsPanel } from './SystemTimingsPanel';
import type { SystemTimingsSnapshot, TelemetryFrame } from '../types';

export const DevToolsApp: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [tick, setTick] = useState(0);
  const [creatureCount, setCreatureCount] = useState(0);
  const [tickRateHz, setTickRateHz] = useState(0);
  const [systemTimings, setSystemTimings] = useState<SystemTimingsSnapshot | undefined>(undefined);

  useEffect(() => {
    if (window.electron?.sendCommand) {
      setIsConnected(true);
    }

    const handleTelemetryUpdate = (telemetry: TelemetryFrame) => {
      setTick(telemetry.tick);
      setCreatureCount(telemetry.creatureCount);
      setTickRateHz(telemetry.tickRateHz);
      setSystemTimings(telemetry.systemTimingsUs);
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

      <IPCHealthPanel timings={systemTimings} />

      <SystemTimingsPanel timings={systemTimings} />
    </div>
  );
};
