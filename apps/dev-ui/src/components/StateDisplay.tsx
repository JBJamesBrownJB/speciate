/**
 * State Display Component
 *
 * Shows current simulation state (tick, creature count, etc.)
 * Updated via Electron IPC state-update events.
 */

import React from 'react';

interface StateDisplayProps {
  tick: number;
  creatureCount: number;
  tickRateHz?: number;
}

export const StateDisplay: React.FC<StateDisplayProps> = ({
  tick,
  creatureCount,
  tickRateHz,
}) => {
  return (
    <div className="section">
      <h2>Simulation State</h2>
      <div className="state-display">
        <pre>
          {JSON.stringify(
            {
              tick,
              creature_count: creatureCount,
              tick_rate_hz: tickRateHz ?? 0,
              frame_protocol: 'MessagePack (stdio)',
            },
            null,
            2
          )}
        </pre>
      </div>
      <p className="info-text">
        State updates received from simulation subprocess via Electron IPC.
      </p>
    </div>
  );
};
