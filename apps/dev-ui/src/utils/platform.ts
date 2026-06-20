/**
 * Platform helpers for the dev-ui.
 *
 * The host OS is supplied by the Electron preload as `window.electron.platform`
 * (Node's process.platform). Used to label Linux-only hardware counters and to
 * gate Windows-only metric panels — never inferred from absent/zeroed telemetry,
 * since Windows may emit zeroed hardware fields rather than omitting them.
 */

export function getPlatform(): string | undefined {
  return typeof window !== 'undefined' ? window.electron?.platform : undefined;
}

export function isWindows(): boolean {
  return getPlatform() === 'win32';
}

export function isLinux(): boolean {
  return getPlatform() === 'linux';
}

/**
 * Whether Linux-only PMU hardware counters can exist on this host. When false,
 * the dev-ui shows a "Linux only" badge instead of the hardware gauges.
 */
export function hardwareCountersSupported(): boolean {
  // Unknown platform (e.g. running outside Electron) -> assume supported so we
  // don't hide data; only the explicitly non-Linux case suppresses the gauges.
  const p = getPlatform();
  return p === undefined || p === 'linux';
}
