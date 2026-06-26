import React, { useState } from 'react';

interface FastForwardControlProps {
  disabled?: boolean;
  onTimeScaleChange?: (scale: number) => void;
}

export const FastForwardControl: React.FC<FastForwardControlProps> = ({
  disabled = false,
  onTimeScaleChange,
}) => {
  const [value, setValue] = useState('1');

  const apply = () => {
    const n = parseFloat(value);
    if (!isFinite(n) || n <= 0) return;
    onTimeScaleChange?.(n);
  };

  return (
    <div className="section">
      <h2>Time Scale</h2>
      <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
        <input
          type="number"
          role="spinbutton"
          min="0.01"
          step="any"
          value={value}
          disabled={disabled}
          onChange={e => setValue(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter') apply(); }}
          style={{ width: '80px' }}
          aria-label="Time scale multiplier"
        />
        <span>×</span>
        <button onClick={apply} disabled={disabled}>Set</button>
      </div>
      <div style={{ fontSize: '11px', color: '#888', marginTop: '4px' }}>
        1 = normal · &gt;1 = fast-forward · max ticks/frame: 5
      </div>
    </div>
  );
};
