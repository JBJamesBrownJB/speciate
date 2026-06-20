import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MetricsColumn } from './MetricsColumn';
import type { SystemTimingsSnapshot, HardwareMetrics, ParallelizationMetrics } from '../types';

// Stub the canvas-heavy children — jsdom has no real 2D context. We only care
// that MetricsColumn renders/omits them, not how they draw.
vi.mock('./VectorizationTachometer', () => ({ VectorizationTachometer: () => <div>gauge:ipc</div> }));
vi.mock('./CacheFirewall', () => ({ CacheFirewall: () => <div>gauge:cache</div> }));
vi.mock('./BranchScope', () => ({ BranchScope: () => <div>gauge:branch</div> }));
vi.mock('./Parallelism', () => ({ Parallelism: () => <div>panel:parallelism</div> }));
vi.mock('./MemoryMetrics', () => ({ MemoryMetrics: () => <div>panel:memory</div> }));
vi.mock('./SystemTimingsPanel', () => ({ SystemTimingsPanel: () => <div>panel:timings</div> }));
vi.mock('./WindowsMetricsPanel', () => ({ WindowsMetricsPanel: () => <div>panel:windows</div> }));

function setPlatform(platform: string | undefined): void {
  (window as unknown as { electron?: { platform?: string } }).electron =
    platform === undefined ? undefined : { platform };
}

const timings = {} as SystemTimingsSnapshot;
const parallel = { processMemoryBytes: 123 } as ParallelizationMetrics;
const hardware = { ipc: 1, l1dMissRate: 0, llcMissRate: 0, branchMissRate: 0 } as HardwareMetrics;

const base = { label: 'LIVE', labelClass: '', tick: 0, creatureCount: 0, tickRateHz: 20 };

afterEach(() => setPlatform(undefined));

describe('MetricsColumn', () => {
  it('on Windows without hardware metrics: shows Linux-only badge AND still renders parallelism + memory', () => {
    setPlatform('win32');
    render(<MetricsColumn {...base} systemTimings={timings} parallelizationMetrics={parallel} />);

    expect(screen.getByText(/linux only/i)).toBeInTheDocument();
    // The regression this guards: these used to be nested inside the hardwareMetrics gate.
    expect(screen.getByText('panel:parallelism')).toBeInTheDocument();
    expect(screen.getByText('panel:memory')).toBeInTheDocument();
    // No gauges without hardware metrics.
    expect(screen.queryByText('gauge:ipc')).not.toBeInTheDocument();
    // No Windows panel unless windowsMetrics.available is set.
    expect(screen.queryByText('panel:windows')).not.toBeInTheDocument();
  });

  it('shows the Windows metrics panel when windowsMetrics is available', () => {
    setPlatform('win32');
    render(
      <MetricsColumn
        {...base}
        systemTimings={timings}
        parallelizationMetrics={parallel}
        windowsMetrics={{
          available: true,
          processCyclesPerSec: 1,
          pageFaultsPerSec: 1,
          pageFaultCount: 1,
          workingSetBytes: 1,
        }}
      />
    );
    expect(screen.getByText('panel:windows')).toBeInTheDocument();
    expect(screen.getByText(/linux only/i)).toBeInTheDocument();
  });

  it('on Windows with a ZEROED hardware metrics object: still shows the badge, not dead gauges', () => {
    // Windows sends a zeroed (truthy) hardwareMetrics object — the badge must
    // still win, otherwise the IPC/cache/branch gauges render with no data.
    setPlatform('win32');
    render(
      <MetricsColumn
        {...base}
        systemTimings={timings}
        parallelizationMetrics={parallel}
        hardwareMetrics={{ ipc: 0, l1dMissRate: 0, llcMissRate: 0, branchMissRate: 0 } as HardwareMetrics}
      />
    );
    expect(screen.getByText(/linux only/i)).toBeInTheDocument();
    expect(screen.queryByText('gauge:ipc')).not.toBeInTheDocument();
    expect(screen.queryByText('gauge:cache')).not.toBeInTheDocument();
  });

  it('with hardware metrics present: renders gauges and no badge', () => {
    setPlatform('linux');
    render(
      <MetricsColumn
        {...base}
        systemTimings={timings}
        parallelizationMetrics={parallel}
        hardwareMetrics={hardware}
      />
    );

    expect(screen.getByText('gauge:ipc')).toBeInTheDocument();
    expect(screen.getByText('gauge:cache')).toBeInTheDocument();
    expect(screen.getByText('gauge:branch')).toBeInTheDocument();
    expect(screen.queryByText(/linux only/i)).not.toBeInTheDocument();
  });
});
