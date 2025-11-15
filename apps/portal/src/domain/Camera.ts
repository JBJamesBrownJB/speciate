import { CAMERA_CONFIG } from "../core/constants";

export interface ITransformable {
  scale: { set(x: number, y?: number): void };
  position: { set(x: number, y: number): void };
}

export class Camera {

  private _x: number;
  private _y: number;
  private _zoom: number;

  constructor(x: number, y: number, zoom: number) {
    this._x = x;
    this._y = y;
    this._zoom = this.clampZoom(zoom);
  }

  get x(): number {
    return this._x;
  }

  get y(): number {
    return this._y;
  }

  get zoom(): number {
    return this._zoom;
  }

  move(x: number, y: number): void {
    this._x = x;
    this._y = y;
  }

  deltaMove(dx: number, dy: number): void {
    this._x += dx;
    this._y += dy;
  }

  setZoom(zoom: number): void {
    this._zoom = this.clampZoom(zoom);
  }

  adjustZoom(factor: number): void {
    this.setZoom(this._zoom * factor);
  }

  worldToScreen(worldX: number, worldY: number): { x: number; y: number } {
    const dx = worldX - this._x;
    const dy = worldY - this._y;

    return {
      x: dx * this._zoom,
      y: dy * this._zoom,
    };
  }

  screenToWorld(screenX: number, screenY: number): { x: number; y: number } {
    const dx = screenX / this._zoom;
    const dy = screenY / this._zoom;

    return {
      x: this._x + dx,
      y: this._y + dy,
    };
  }

  applyTransform(
    container: ITransformable,
    screenWidth: number,
    screenHeight: number
  ): void {
    container.scale.set(this._zoom);

    container.position.set(
      screenWidth / 2 - this._x * this._zoom,
      screenHeight / 2 - this._y * this._zoom
    );
  }

  private clampZoom(zoom: number): number {
    return Math.max(CAMERA_CONFIG.MIN_ZOOM, Math.min(CAMERA_CONFIG.MAX_ZOOM, zoom));
  }
}
