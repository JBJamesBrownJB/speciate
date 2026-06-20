import { describe, it, expect, afterEach } from 'vitest';
import { isWindows, isLinux, hardwareCountersSupported, getPlatform } from './platform';

function setPlatform(platform: string | undefined): void {
  (window as unknown as { electron?: { platform?: string } }).electron =
    platform === undefined ? undefined : { platform };
}

describe('platform helpers', () => {
  afterEach(() => setPlatform(undefined));

  it('reads platform from window.electron', () => {
    setPlatform('win32');
    expect(getPlatform()).toBe('win32');
    expect(isWindows()).toBe(true);
    expect(isLinux()).toBe(false);
  });

  it('detects linux', () => {
    setPlatform('linux');
    expect(isLinux()).toBe(true);
    expect(isWindows()).toBe(false);
  });

  it('reports hardware counters available on linux and unknown, but not windows', () => {
    setPlatform('linux');
    expect(hardwareCountersSupported()).toBe(true);
    setPlatform(undefined);
    expect(hardwareCountersSupported()).toBe(true); // unknown -> don't hide data
    setPlatform('win32');
    expect(hardwareCountersSupported()).toBe(false);
  });
});
