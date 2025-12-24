/**
 * DNA Settings Component
 *
 * Unified DNA configuration panel that applies to both manual spawning and trial loading.
 * When randomize is ON, each creature gets unique random DNA.
 */

import React from 'react';

// Gene expression constants (must match Rust constants)
const SIZE_MIN = 0.1;
const SIZE_MAX = 10.0;
const FOV_MIN = 45.0;
const FOV_MAX = 340.0;
export const DEFAULT_SIZE_GENE = 0.09;
export const DEFAULT_FOV_GENE = 0.46;

function expressGene(gene: number, min: number, max: number): number {
  return min + Math.min(1, Math.max(0, gene)) * (max - min);
}

interface DnaSettingsProps {
  sizeGene: number;
  fovGene: number;
  randomize: boolean;
  onSizeChange: (value: number) => void;
  onFovChange: (value: number) => void;
  onRandomizeChange: (value: boolean) => void;
  onReset: () => void;
  disabled?: boolean;
}

export const DnaSettings: React.FC<DnaSettingsProps> = ({
  sizeGene,
  fovGene,
  randomize,
  onSizeChange,
  onFovChange,
  onRandomizeChange,
  onReset,
  disabled,
}) => {
  const expressedSize = expressGene(sizeGene, SIZE_MIN, SIZE_MAX);
  const expressedFov = expressGene(fovGene, FOV_MIN, FOV_MAX);

  return (
    <div className="section">
      <h2>DNA Settings</h2>
      <p className="info-text">
        These settings apply to both manual spawning and trial loading.
      </p>

      <div className="form-group">
        <label htmlFor="size-gene">
          Size Gene: {(sizeGene * 100).toFixed(0)}%
          <span className="phenotype-preview"> = {expressedSize.toFixed(2)}m</span>
        </label>
        <input
          id="size-gene"
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={sizeGene}
          onChange={(e) => onSizeChange(Number(e.target.value))}
          disabled={disabled || randomize}
        />
        <div className="slider-labels">
          <span>{SIZE_MIN}m</span>
          <span>{SIZE_MAX}m</span>
        </div>
      </div>

      <div className="form-group">
        <label htmlFor="fov-gene">
          FOV Gene: {(fovGene * 100).toFixed(0)}%
          <span className="phenotype-preview"> = {expressedFov.toFixed(0)}°</span>
        </label>
        <input
          id="fov-gene"
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={fovGene}
          onChange={(e) => onFovChange(Number(e.target.value))}
          disabled={disabled || randomize}
        />
        <div className="slider-labels">
          <span>{FOV_MIN}°</span>
          <span>{FOV_MAX}°</span>
        </div>
      </div>

      <div className="form-group checkbox-group">
        <label>
          <input
            type="checkbox"
            checked={randomize}
            onChange={(e) => onRandomizeChange(e.target.checked)}
            disabled={disabled}
          />
          Randomize DNA (each creature gets unique random values)
        </label>
      </div>

      <button type="button" onClick={onReset} disabled={disabled || randomize}>
        Reset to Defaults
      </button>
    </div>
  );
};
