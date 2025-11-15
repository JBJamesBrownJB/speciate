declare global {
  interface Window {
    electron?: {
      onStateUpdate: (callback: (state: import('./types/GameState').GameState) => void) => void;

      removeStateUpdateListener: () => void;

      getLatestState: () => Promise<import('./types/GameState').GameState | null>;
    };
  }
}

export {};
