import { CAMERA_CONFIG } from "../core/constants";

/**
 * Interface for objects that can have camera transforms applied to them.
 * This provides a domain boundary - Camera stays in domain layer but can work
 * with rendering layer objects through this interface.
 */
export interface ITransformable {
  scale: { set(x: number, y?: number): void };
  position: { set(x: number, y: number): void };
}

/**
 * Camera represents the viewport into the simulation world.
 *
 * Coordinates:
 * - World space: meters (f32), can range from -1,000,000 to +1,000,000
 * - Screen space: pixels
 *
 * Zoom:
 * - Measured in pixels per meter
 * - Min: 0.0005 px/m (max zoomed out - can view full 2000km × 2000km world)
 * - Max: 200 px/m (max zoomed in - 1 meter = 200 pixels)
 */
export class Camera {

  private _x: number;
  private _y: number;
  private _zoom: number;

  /**
   * Create a new camera
   * @param x Camera X position in world space (meters)
   * @param y Camera Y position in world space (meters)
   * @param zoom Zoom level in pixels per meter (1-100)
   */
  constructor(x: number, y: number, zoom: number) {
    this._x = x;
    this._y = y;
    this._zoom = this.clampZoom(zoom);
  }

  /**
   * Current X position of camera in world space (meters)
   */
  get x(): number {
    return this._x;
  }

  /**
   * Current Y position of camera in world space (meters)
   */
  get y(): number {
    return this._y;
  }

  /**
   * Current zoom level (pixels per meter)
   */
  get zoom(): number {
    return this._zoom;
  }

  /**
   * Move camera to an absolute position in world space
   * @param x World X coordinate (meters)
   * @param y World Y coordinate (meters)
   */
  move(x: number, y: number): void {
    this._x = x;
    this._y = y;
  }

  /**
   * Move camera by a relative amount
   * @param dx Delta X (meters)
   * @param dy Delta Y (meters)
   */
  deltaMove(dx: number, dy: number): void {
    this._x += dx;
    this._y += dy;
  }

  /**
   * Set zoom level (clamped to valid range)
   * @param zoom Zoom level in pixels per meter (1-100)
   */
  setZoom(zoom: number): void {
    this._zoom = this.clampZoom(zoom);
  }

  /**
   * Adjust zoom by a multiplicative factor
   * @param factor Zoom multiplier (e.g., 2.0 = double zoom, 0.5 = half zoom)
   */
  adjustZoom(factor: number): void {
    this.setZoom(this._zoom * factor);
  }

  /**
   * Convert world coordinates to screen coordinates
   * @param worldX World X coordinate (meters)
   * @param worldY World Y coordinate (meters)
   * @returns Screen coordinates (pixels)
   */
  worldToScreen(worldX: number, worldY: number): { x: number; y: number } {
    const dx = worldX - this._x;
    const dy = worldY - this._y;

    return {
      x: dx * this._zoom,
      y: dy * this._zoom,
    };
  }

  /**
   * Convert screen coordinates to world coordinates
   * @param screenX Screen X coordinate (pixels)
   * @param screenY Screen Y coordinate (pixels)
   * @returns World coordinates (meters)
   */
  screenToWorld(screenX: number, screenY: number): { x: number; y: number } {
    const dx = screenX / this._zoom;
    const dy = screenY / this._zoom;

    return {
      x: this._x + dx,
      y: this._y + dy,
    };
  }

  /**
   * Apply camera transform to a container (world container pattern).
   * This sets the container's scale and position to display the world
   * with the camera's zoom level and position, centered on screen.
   *
   * @param container Object with scale and position properties (e.g., Pixi.js Container)
   * @param screenWidth Screen width in pixels
   * @param screenHeight Screen height in pixels
   */
  applyTransform(
    container: ITransformable,
    screenWidth: number,
    screenHeight: number
  ): void {
    // Apply zoom as uniform scale (preserves aspect ratio)
    container.scale.set(this._zoom);

    // Position container to center camera position on screen
    // Formula: center of screen - (camera world position * zoom)
    container.position.set(
      screenWidth / 2 - this._x * this._zoom,
      screenHeight / 2 - this._y * this._zoom
    );
  }

  /**
   * Clamp zoom to valid range [MIN_ZOOM, MAX_ZOOM]
   */
  private clampZoom(zoom: number): number {
    return Math.max(CAMERA_CONFIG.MIN_ZOOM, Math.min(CAMERA_CONFIG.MAX_ZOOM, zoom));
  }
}
