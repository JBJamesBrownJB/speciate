import type { EntityMap, EntityState } from '../types/entity';

export class EntityStateManager {
  private entities: EntityMap = new Map();

  updateEntity(state: EntityState): void {
    const existing = this.entities.get(state.id);

    if (existing) {
      this.entities.set(state.id, {
        ...state,
        previousPosition: existing.position,
        previousOrientation: existing.orientation,
        lastUpdateTime: performance.now(),
      });
    } else {
      this.entities.set(state.id, {
        ...state,
        previousPosition: state.position,
        previousOrientation: state.orientation,
        lastUpdateTime: performance.now(),
      });
    }
  }

  removeEntity(id: string): void {
    this.entities.delete(id);
  }

  getEntities(): EntityMap {
    return this.entities;
  }

  clear(): void {
    this.entities.clear();
  }

  getEntityCount(): number {
    return this.entities.size;
  }
}
