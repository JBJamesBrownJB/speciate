import type { SimulationStateMessage } from '@/types/messages';

export class StateManager {
  private currentTick: number = 0;

  public updateFromServer(message: SimulationStateMessage): void {
    // Simply track the current tick for HUD display
    this.currentTick = message.tick;
  }

  public getCurrentTick(): number {
    return this.currentTick;
  }

  public clear(): void {
    this.currentTick = 0;
  }
}
