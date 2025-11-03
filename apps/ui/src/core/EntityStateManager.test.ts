import { describe, it, expect, beforeEach } from 'vitest';
import { EntityStateManager } from './EntityStateManager';
import type { EntityState } from '../types/entity';

describe('EntityStateManager', () => {
  let manager: EntityStateManager;

  beforeEach(() => {
    manager = new EntityStateManager();
  });

  it('should add new entity with initial state', () => {
    const state: EntityState = {
      id: 'entity1',
      position: { x: 100, y: 100 },
      orientation: 0,
      radius: 10,
    };

    manager.updateEntity(state);

    const entities = manager.getEntities();
    const entity = entities.get('entity1');

    expect(entity).toBeDefined();
    expect(entity?.position).toEqual({ x: 100, y: 100 });
    expect(entity?.previousPosition).toEqual({ x: 100, y: 100 });
  });

  it('should update existing entity and preserve previous state', () => {
    const initialState: EntityState = {
      id: 'entity1',
      position: { x: 100, y: 100 },
      orientation: 0,
      radius: 10,
    };

    manager.updateEntity(initialState);

    const updatedState: EntityState = {
      id: 'entity1',
      position: { x: 200, y: 200 },
      orientation: Math.PI / 2,
      radius: 10,
    };

    manager.updateEntity(updatedState);

    const entity = manager.getEntities().get('entity1');
    expect(entity?.position).toEqual({ x: 200, y: 200 });
    expect(entity?.previousPosition).toEqual({ x: 100, y: 100 });
    expect(entity?.orientation).toBe(Math.PI / 2);
    expect(entity?.previousOrientation).toBe(0);
  });

  it('should remove entity', () => {
    const state: EntityState = {
      id: 'entity1',
      position: { x: 100, y: 100 },
      orientation: 0,
      radius: 10,
    };

    manager.updateEntity(state);
    manager.removeEntity('entity1');

    expect(manager.getEntities().has('entity1')).toBe(false);
  });

  it('should clear all entities', () => {
    manager.updateEntity({
      id: 'entity1',
      position: { x: 100, y: 100 },
      orientation: 0,
      radius: 10,
    });

    manager.updateEntity({
      id: 'entity2',
      position: { x: 200, y: 200 },
      orientation: 0,
      radius: 10,
    });

    manager.clear();

    expect(manager.getEntityCount()).toBe(0);
  });

  it('should return correct entity count', () => {
    expect(manager.getEntityCount()).toBe(0);

    manager.updateEntity({
      id: 'entity1',
      position: { x: 100, y: 100 },
      orientation: 0,
      radius: 10,
    });

    expect(manager.getEntityCount()).toBe(1);
  });
});
