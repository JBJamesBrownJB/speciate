import { Graphics, Container } from 'pixi.js';

const GRID_LINE_COLOR = 0x444444;
const GRID_LINE_ALPHA = 0.6;

export class SpatialGridOverlay {
  private graphics: Graphics;
  private visible: boolean = false;
  private cellSize: number = 50;

  constructor(container: Container) {
    this.graphics = new Graphics();
    this.graphics.visible = false;
    container.addChild(this.graphics);
  }

  setCellSize(cellSize: number): void {
    this.cellSize = cellSize;
  }

  toggle(): void {
    this.visible = !this.visible;
    this.graphics.visible = this.visible;
  }

  show(): void {
    this.visible = true;
    this.graphics.visible = true;
  }

  hide(): void {
    this.visible = false;
    this.graphics.visible = false;
  }

  isVisible(): boolean {
    return this.visible;
  }

  update(
    cameraX: number,
    cameraY: number,
    zoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): void {
    if (!this.visible) return;

    this.render(cameraX, cameraY, zoom, viewportWidth, viewportHeight);
  }

  destroy(): void {
    this.graphics.destroy();
  }

  private render(
    cameraX: number,
    cameraY: number,
    zoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): void {
    this.graphics.clear();

    const halfViewW = (viewportWidth / 2) / zoom;
    const halfViewH = (viewportHeight / 2) / zoom;

    const worldLeft = cameraX - halfViewW;
    const worldRight = cameraX + halfViewW;
    const worldTop = cameraY - halfViewH;
    const worldBottom = cameraY + halfViewH;

    const startCellX = Math.floor(worldLeft / this.cellSize);
    const endCellX = Math.ceil(worldRight / this.cellSize);
    const startCellY = Math.floor(worldTop / this.cellSize);
    const endCellY = Math.ceil(worldBottom / this.cellSize);

    const lineWidth = 1.0 / zoom;

    this.graphics.setStrokeStyle({
      width: lineWidth,
      color: GRID_LINE_COLOR,
      alpha: GRID_LINE_ALPHA,
    });

    for (let cellX = startCellX; cellX <= endCellX; cellX++) {
      const x = cellX * this.cellSize;
      this.graphics.moveTo(x, worldTop);
      this.graphics.lineTo(x, worldBottom);
    }

    for (let cellY = startCellY; cellY <= endCellY; cellY++) {
      const y = cellY * this.cellSize;
      this.graphics.moveTo(worldLeft, y);
      this.graphics.lineTo(worldRight, y);
    }

    this.graphics.stroke();
  }
}
