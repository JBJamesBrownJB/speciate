import { Camera } from './Camera';
import { Creature } from './Creature';
import { SpatialQuery } from '@/utils/SpatialQuery';

export interface WorldBounds {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

export class Viewport {
  private _width: number;
  private _height: number;

  constructor(width: number, height: number) {
    this._width = width;
    this._height = height;
  }

  get width(): number {
    return this._width;
  }

  get height(): number {
    return this._height;
  }

  resize(width: number, height: number): void {
    this._width = width;
    this._height = height;
  }

  getWorldBounds(camera: Camera): WorldBounds {
    const halfWidthWorld = (this._width / 2) / camera.zoom;
    const halfHeightWorld = (this._height / 2) / camera.zoom;

    return {
      minX: camera.x - halfWidthWorld,
      maxX: camera.x + halfWidthWorld,
      minY: camera.y - halfHeightWorld,
      maxY: camera.y + halfHeightWorld
    };
  }

  isCreatureVisible(creature: Creature, camera: Camera): boolean {
    const worldBounds = this.getWorldBounds(camera);
    return SpatialQuery.isInViewport(creature, worldBounds);
  }

  cullCreatures(creatures: Creature[], camera: Camera): Creature[] {
    return creatures.filter(creature => this.isCreatureVisible(creature, camera));
  }
}
