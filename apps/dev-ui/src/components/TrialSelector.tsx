/**
 * Trial Selector Component
 *
 * Load predefined trial templates for regression testing.
 * Sends DevLoadTrial command to simulation via Electron IPC.
 */

import React, { useState } from 'react';

interface TrialSelectorProps {
  onLoadTrial: (template: string) => void;
  disabled?: boolean;
}

const TRIAL_TEMPLATES = [
  {
    name: 'default-spawn-baseline',
    label: 'Default Spawn Baseline',
    description: '10×10 grid at world center (100 creatures)',
  },
  {
    name: 'crowd-navigation',
    label: 'Crowd Navigation Test',
    description: '200 static obstacles + 50 mobile seekers',
  },
];

export const TrialSelector: React.FC<TrialSelectorProps> = ({
  onLoadTrial,
  disabled,
}) => {
  const [selectedTrial, setSelectedTrial] = useState<string>(
    TRIAL_TEMPLATES[0].name
  );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onLoadTrial(selectedTrial);
  };

  const selectedTemplate = TRIAL_TEMPLATES.find(
    (t) => t.name === selectedTrial
  );

  return (
    <div className="section">
      <h2>Load Trial Template</h2>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="trial-select">Trial Template</label>
          <select
            id="trial-select"
            value={selectedTrial}
            onChange={(e) => setSelectedTrial(e.target.value)}
            disabled={disabled}
          >
            {TRIAL_TEMPLATES.map((trial) => (
              <option key={trial.name} value={trial.name}>
                {trial.label}
              </option>
            ))}
          </select>
        </div>

        {selectedTemplate && (
          <p className="info-text">{selectedTemplate.description}</p>
        )}

        <button type="submit" disabled={disabled} style={{ marginTop: '12px' }}>
          Load Trial
        </button>

        <p className="info-text" style={{ marginTop: '12px' }}>
          Trial templates are TOML files in apps/simulation/trials/. They define
          spawn patterns for reproducible regression tests.
        </p>
      </form>
    </div>
  );
};
