import type { IPCClient } from './IPCClient';
import { ElectronIPCClient } from './ElectronIPCClient';

function detectEnvironment(): 'electron' | 'browser' {
  if (typeof window === 'undefined') {
    return 'browser';
  }

  if ('electron' in window) {
    return 'electron';
  }

  return 'browser';
}

export function createIPCClient(): IPCClient | null {
  const env = detectEnvironment();

  switch (env) {
    case 'electron':
      return new ElectronIPCClient();

    case 'browser':
      return null;
  }
}

export type { IPCClient } from './IPCClient';
export { ElectronIPCClient } from './ElectronIPCClient';
