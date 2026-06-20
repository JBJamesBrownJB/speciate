/**
 * Trial Selector Component
 *
 * Load predefined trial templates for regression testing.
 * Sends DevLoadTrial command to simulation via Electron IPC.
 *
 * Trial templates are auto-discovered from apps/simulation/trials/*.toml at build time.
 * See: scripts/generate-trial-list.cjs
 */

import React, { useState } from 'react';
import { TRIAL_TEMPLATES } from '../generated/trial-templates';

interface TrialSelectorProps {
  onLoadTrial: (template: string) => void;
  disabled?: boolean;
  randomizeDna?: boolean;
}

export const TrialSelector: React.FC<TrialSelectorProps> = ({
  onLoadTrial,
  disabled,
  randomizeDna,
}) => {
  const [selectedTrial, setSelectedTrial] = useState<string>(
    TRIAL_TEMPLATES[0]?.name ?? ''
  );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onLoadTrial(selectedTrial);
  };

  if (TRIAL_TEMPLATES.length === 0) {
    return (
      <div className="section">
        <h2>Load Trial Template</h2>
        <p className="info-text">
          No trial templates generated. Run <code>npm run generate:trials</code> in
          apps/dev-ui (it scans apps/simulation/specs/**/*.toml).
        </p>
      </div>
    );
  }

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

        {randomizeDna && (
          <p className="info-text" style={{ marginTop: '12px', fontWeight: 'bold' }}>
            🎲 Randomize DNA is ON - each creature will get unique random DNA.
          </p>
        )}

        <p className="info-text" style={{ marginTop: '12px' }}>
          Trial templates are TOML files in apps/simulation/trials/. DNA settings
          above apply to loaded trials.
        </p>
      </form>
    </div>
  );
};
