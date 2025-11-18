declare global {
  interface Window {
    electron?: {
      onStateUpdateBinary: (callback: (binaryData: Uint8Array) => void) => void;

      removeStateUpdateListener: () => void;

      getLatestState: () => Promise<import('./types/GameState').GameState | null>;

      sendCommand: (command: { type: string; [key: string]: unknown }) => void;
    };
  }
}

export {};
