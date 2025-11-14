import type { IPCClient } from './IPCClient';
import { ElectronIPCClient } from './ElectronIPCClient';

/**
 * Runtime environment detection
 * @returns 'electron' | 'browser'
 */
function detectEnvironment(): 'electron' | 'browser' {
  if (typeof window === 'undefined') {
    return 'browser';
  }

  // Electron detection (window.electron injected by preload script)
  if ('electron' in window) {
    return 'electron';
  }

  return 'browser';
}

/**
 * Create appropriate IPC client for current environment
 *
 * Factory function auto-detects runtime (Electron/browser)
 * and returns the correct implementation.
 *
 * @returns IPCClient instance or null (browser mode)
 */
export function createIPCClient(): IPCClient | null {
  const env = detectEnvironment();

  switch (env) {
    case 'electron':
      return new ElectronIPCClient();

    case 'browser':
      return null;
  }
}

// Re-export interface and implementations for external use
export type { IPCClient } from './IPCClient';
export { ElectronIPCClient } from './ElectronIPCClient';
