import { Camera } from './Camera';
import { Creature } from './Creature';
import { SpatialQuery } from '@/utils/SpatialQuery';

/**
 * World bounds in meters
 */
export interface WorldBounds {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

/**
 * Viewport represents the screen area showing the simulation.
 *
 * It handles:
 * - Calculating which part of the world is visible
 * - Culling creatures outside the viewport
 * - Converting between screen and world coordinates (via Camera)
 */
export class Viewport {
  private _width: number;
  private _height: number;

  /**
   * Create a new viewport
   * @param width Width in pixels
   * @param height Height in pixels
   */
  constructor(width: number, height: number) {
    this._width = width;
    this._height = height;
  }

  /**
   * Viewport width in pixels
   */
  get width(): number {
    return this._width;
  }

  /**
   * Viewport height in pixels
   */
  get height(): number {
    return this._height;
  }

  /**
   * Update viewport dimensions (e.g., on window resize)
   * @param width New width in pixels
   * @param height New height in pixels
   */
  resize(width: number, height: number): void {
    this._width = width;
    this._height = height;
  }

  /**
   * Calculate the world bounds visible in this viewport
   * @param camera Current camera state
   * @returns World bounds in meters
   */
  getWorldBounds(camera: Camera): WorldBounds {
    // Calculate viewport dimensions in world units
    // Viewport center IS the camera position, so we calculate bounds directly
    const halfWidthWorld = (this._width / 2) / camera.zoom;
    const halfHeightWorld = (this._height / 2) / camera.zoom;

    return {
      minX: camera.x - halfWidthWorld,
      maxX: camera.x + halfWidthWorld,
      minY: camera.y - halfHeightWorld,
      maxY: camera.y + halfHeightWorld
    };
  }

  /**
   * Check if a creature is visible in the viewport
   * @param creature Creature to check
   * @param camera Current camera state
   * @returns True if creature (or part of it) is visible
   */
  isCreatureVisible(creature: Creature, camera: Camera): boolean {
    const worldBounds = this.getWorldBounds(camera);
    return SpatialQuery.isInViewport(creature, worldBounds);
  }

  /**
   * Filter creatures to only those visible in the viewport
   * @param creatures All creatures
   * @param camera Current camera state
   * @returns Only visible creatures
   */
  cullCreatures(creatures: Creature[], camera: Camera): Creature[] {
    return creatures.filter(creature => this.isCreatureVisible(creature, camera));
  }
}
