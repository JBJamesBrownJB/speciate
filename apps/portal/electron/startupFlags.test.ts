import { describe, it, expect } from 'vitest';
// @ts-ignore - plain CJS module under test; types not needed for this spec
import { sandboxWorkaroundSwitches } from './startupFlags.cjs';

/**
 * The no-sandbox / dev-shm switches exist ONLY to work around Linux SUID/ESRCH
 * crashes. Applying them elsewhere needlessly weakens the Windows/macOS builds.
 */
describe('sandboxWorkaroundSwitches', () => {
  it('returns the Linux workaround switches on linux', () => {
    const switches = sandboxWorkaroundSwitches('linux').map(([name]: [string]) => name);
    expect(switches).toContain('no-sandbox');
    expect(switches).toContain('disable-gpu-sandbox');
    expect(switches).toContain('disable-dev-shm-usage');
  });

  it('returns no sandbox-weakening switches on win32', () => {
    expect(sandboxWorkaroundSwitches('win32')).toEqual([]);
  });

  it('returns no sandbox-weakening switches on darwin', () => {
    expect(sandboxWorkaroundSwitches('darwin')).toEqual([]);
  });
});
