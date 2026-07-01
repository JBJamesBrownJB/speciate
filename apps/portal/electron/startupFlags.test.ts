import { describe, it, expect } from 'vitest';
// @ts-ignore - plain CJS module under test; types not needed for this spec
import { sandboxWorkaroundSwitches, shouldOpenDevTools } from './startupFlags.cjs';

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

/**
 * The dev-tools window is opt-in via --dev-tools (as apps/portal/AGENTS.md has
 * always documented for `npm run dev:tools`) — plain `npm run dev` is just the
 * game window.
 */
describe('shouldOpenDevTools', () => {
  it('opens only when dev mode AND the --dev-tools flag are both present', () => {
    expect(shouldOpenDevTools(true, ['electron', '.', '--dev-tools'])).toBe(true);
  });

  it('does not open without the flag', () => {
    expect(shouldOpenDevTools(true, ['electron', '.'])).toBe(false);
  });

  it('never opens in production, even with the flag', () => {
    expect(shouldOpenDevTools(false, ['app', '--dev-tools'])).toBe(false);
  });
});
