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

export const DevToolsApp: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [tick, setTick] = useState(0);
  const [creatureCount, setCreatureCount] = useState(0);

  useEffect(() => {
    // Check if Electron IPC is available
    if (window.electron?.sendCommand) {
      setIsConnected(true);
    }

    // Listen for state updates from simulation
    const handleStateUpdate = (state: any) => {
      setTick(state.tick || 0);
      setCreatureCount(state.creatures?.length || 0);
    };

    window.electron?.onStateUpdate?.(handleStateUpdate);

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

      <StateDisplay tick={tick} creatureCount={creatureCount} />
    </div>
  );
};
