import type { Position } from '../types/entity';

export interface ViewportBounds {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

export class ViewportCuller {
  constructor(private padding: number) {}

  calculateBounds(
    canvasWidth: number,
    canvasHeight: number,
    cameraX: number,
    cameraY: number
  ): ViewportBounds {
    return {
      minX: cameraX - this.padding,
      maxX: cameraX + canvasWidth + this.padding,
      minY: cameraY - this.padding,
      maxY: cameraY + canvasHeight + this.padding,
    };
  }

  isVisible(position: Position, radius: number, bounds: ViewportBounds): boolean {
    return (
      position.x + radius >= bounds.minX &&
      position.x - radius <= bounds.maxX &&
      position.y + radius >= bounds.minY &&
      position.y - radius <= bounds.maxY
    );
  }
}
