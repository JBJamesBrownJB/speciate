import type { TelemetryFrame } from './types/TelemetryFrame';

declare global {
  interface Window {
    electron?: {
      /** @deprecated Use onNAPIBufferUpdate instead */
      onStateUpdateBinary: (callback: (binaryData: Uint8Array) => void) => () => void;

      onNAPIBufferUpdate: (callback: (data: { buffer: number[], creatureCount: number }) => void) => () => void;

      onTelemetryUpdate: (callback: (telemetry: TelemetryFrame) => void) => () => void;

      removeStateUpdateListener: () => void;

      getLatestState: () => Promise<import('./types/GameState').GameState | null>;

      sendCommand: (command: { type: string; [key: string]: unknown }) => void;

      selectCreatureDebug: (creatureId: number | null) => void;

      onPerceptionDebugUpdate: (callback: (buffer: Float32Array) => void) => () => void;

      setPaused: (paused: boolean) => void;

      setTimeScale: (scale: number) => void;

      setViewportBounds: (bounds: {
        minX: number;
        minY: number;
        maxX: number;
        maxY: number;
        margin: number;
      }) => void;

      queryL1Cell: (worldX: number, worldY: number) => Promise<{
        cellX: number;
        cellY: number;
        creatureCount: number;
        totalMass: number;
        maxSize: number;
        avgSize: number;
      } | null>;
    };
  }
}

export {};
