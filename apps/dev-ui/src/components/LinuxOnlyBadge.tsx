/**
 * LinuxOnlyBadge
 *
 * Shown in place of the hardware-counter gauges on non-Linux hosts. The PMU
 * counters (IPC, cache, branch) are backed by perf_event_open and have no
 * user-space Windows equivalent — so we keep the slot visible and labelled
 * rather than silently hiding it. See docs/scale/windows-parity-strategy.md §4.
 */

import React from 'react';

interface Props {
  label?: string;
}

export const LinuxOnlyBadge: React.FC<Props> = ({ label = 'Hardware Counters' }) => {
  return (
    <div className="panel linux-only-badge" role="note">
      <div className="panel-header">
        {label} <span className="badge-tag">Linux only</span>
      </div>
      <div className="panel-content">
        <p className="no-data">IPC · cache · branch counters are not available on this OS.</p>
        <p className="hint">
          Backed by perf_event_open (Linux kernel); no Windows user-space equivalent.
        </p>
      </div>
    </div>
  );
};

export default LinuxOnlyBadge;
