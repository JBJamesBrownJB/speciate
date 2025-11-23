import type { TelemetryFrame } from './types/TelemetryFrame';

declare global {
  interface Window {
    electron?: {
      /** @deprecated Use onNAPIBufferUpdate instead */
      onStateUpdateBinary: (callback: (binaryData: Uint8Array) => void) => void;

      onNAPIBufferUpdate: (callback: (data: { buffer: number[], creatureCount: number }) => void) => void;

      onTelemetryUpdate: (callback: (telemetry: TelemetryFrame) => void) => void;

      removeStateUpdateListener: () => void;

      getLatestState: () => Promise<import('./types/GameState').GameState | null>;

      sendCommand: (command: { type: string; [key: string]: unknown }) => void;
    };
  }
}

export {};
