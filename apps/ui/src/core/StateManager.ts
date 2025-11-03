import type { SimulationStateMessage } from '@/types/messages';
import type { EntityState, Vec3 } from '@/types/entities';
import { lerpVec3, calculateInterpolationFactor } from '@/utils/math';

export class StateManager {
  private entityStates: Map<string, EntityState> = new Map();
  private currentTick: number = 0;
  private readonly SERVER_UPDATE_INTERVAL = 100;

  public updateFromServer(message: SimulationStateMessage): void {
    const entityId = 'main_entity';
    const existingState = this.entityStates.get(entityId);
    const newPosition: Vec3 = {
      x: message.entity.x,
      y: message.entity.y,
      z: message.entity.z,
    };

    if (existingState) {
      const updatedState: EntityState = {
        ...existingState,
        prevPosition: existingState.currentPosition,
        currentPosition: newPosition,
        prevTimestamp: existingState.currentTimestamp,
        currentTimestamp: Date.now(),
        tick: message.tick,
      };
      this.entityStates.set(entityId, updatedState);
    } else {
      const now = Date.now();
      const initialState: EntityState = {
        id: entityId,
        prevPosition: newPosition,
        currentPosition: newPosition,
        prevTimestamp: now,
        currentTimestamp: now,
        tick: message.tick,
      };
      this.entityStates.set(entityId, initialState);
    }
    this.currentTick = message.tick;
  }

  public getInterpolatedPosition(entityId: string = 'main_entity'): Vec3 | null {
    const state = this.entityStates.get(entityId);
    if (!state) return null;
    const now = Date.now();
    const t = calculateInterpolationFactor(
      now,
      state.currentTimestamp,
      this.SERVER_UPDATE_INTERVAL
    );
    return lerpVec3(state.prevPosition, state.currentPosition, t);
  }

  public getEntityIds(): string[] {
    return Array.from(this.entityStates.keys());
  }

  public getCurrentTick(): number {
    return this.currentTick;
  }

  public clear(): void {
    this.entityStates.clear();
    this.currentTick = 0;
  }
}
