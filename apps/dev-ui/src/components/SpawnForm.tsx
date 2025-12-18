/**
 * Spawn Form Component
 *
 * Manual creature spawning interface with position controls.
 * DNA settings are managed by the DnaSettings component.
 * Sends DevSpawnCreature command to simulation via Electron IPC.
 */

import React, { useState, useCallback } from 'react';

interface SpawnFormProps {
  onSpawn: (x: number, y: number) => void;
  disabled?: boolean;
}

export const SpawnForm: React.FC<SpawnFormProps> = ({ onSpawn, disabled }) => {
  const [x, setX] = useState<number>(0);
  const [y, setY] = useState<number>(0);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      onSpawn(x, y);
    },
    [x, y, onSpawn]
  );

  return (
    <div className="section">
      <h2>Spawn Creature</h2>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="spawn-x">X Position (meters)</label>
          <input
            id="spawn-x"
            type="number"
            value={x}
            onChange={(e) => setX(Number(e.target.value))}
            step="10"
            disabled={disabled}
          />
        </div>

        <div className="form-group">
          <label htmlFor="spawn-y">Y Position (meters)</label>
          <input
            id="spawn-y"
            type="number"
            value={y}
            onChange={(e) => setY(Number(e.target.value))}
            step="10"
            disabled={disabled}
          />
        </div>

        <button type="submit" disabled={disabled}>
          Spawn Creature
        </button>
      </form>
    </div>
  );
};
