import type { GameState } from '../../types/GameState';

export interface IPCClient {
  connect(): Promise<void>;

  onStateUpdate(callback: (state: GameState) => void): () => void;

  getLatestState(): GameState | null;

  disconnect(): Promise<void>;
}
