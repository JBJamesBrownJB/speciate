import type { GameState, PerceptionDebugData } from '../../types/GameState';
import type { TelemetryFrame } from '../../types/TelemetryFrame';

export interface IPCClient {
  connect(): Promise<void>;

  onStateUpdate(callback: (state: GameState) => void): () => void;

  onTelemetryUpdate(callback: (telemetry: TelemetryFrame) => void): () => void;

  onPerceptionDebugUpdate(callback: (data: PerceptionDebugData | null) => void): () => void;

  getLatestState(): GameState | null;

  disconnect(): Promise<void>;

  selectCreatureDebug(creatureId: number | null): void;
}
