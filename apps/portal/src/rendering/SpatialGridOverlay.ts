import { Graphics, Container } from 'pixi.js';
import type { QueriedCell } from '@/types/GameState';

const GRID_LINE_COLOR = 0x444444;
const GRID_LINE_ALPHA = 0.6;
const QUERIED_CELL_COLOR = 0x22aa22;
const QUERIED_CELL_ALPHA = 0.25;
const CREATURE_CELL_COLOR = 0xdddd22;
const CREATURE_CELL_ALPHA = 0.35;

export class SpatialGridOverlay {
  private graphics: Graphics;
  private cellGraphics: Graphics;
  private visible: boolean = false;
  private cellSize: number = 50;
  private queriedCells: QueriedCell[] = [];
  private creatureCell: QueriedCell | null = null;

  constructor(container: Container) {
    this.cellGraphics = new Graphics();
    this.cellGraphics.visible = false;
    container.addChild(this.cellGraphics);

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
    this.cellGraphics.visible = this.visible;
  }

  show(): void {
    this.visible = true;
    this.graphics.visible = true;
    this.cellGraphics.visible = true;
  }

  hide(): void {
    this.visible = false;
    this.graphics.visible = false;
    this.cellGraphics.visible = false;
  }

  updateQueriedCells(queriedCells: QueriedCell[], creatureCell: QueriedCell): void {
    this.queriedCells = queriedCells;
    this.creatureCell = creatureCell;
  }

  clearQueriedCells(): void {
    this.queriedCells = [];
    this.creatureCell = null;
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
    this.cellGraphics.destroy();
  }

  private render(
    cameraX: number,
    cameraY: number,
    zoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): void {
    this.graphics.clear();
    this.cellGraphics.clear();

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

    // Render queried cells (green fill)
    if (this.queriedCells.length > 0) {
      for (const cell of this.queriedCells) {
        const worldX = cell.x * this.cellSize;
        const worldY = cell.y * this.cellSize;
        this.cellGraphics.rect(worldX, worldY, this.cellSize, this.cellSize);
      }
      this.cellGraphics.fill({ color: QUERIED_CELL_COLOR, alpha: QUERIED_CELL_ALPHA });
    }

    // Render creature's cell (yellow fill) on top
    if (this.creatureCell) {
      const worldX = this.creatureCell.x * this.cellSize;
      const worldY = this.creatureCell.y * this.cellSize;
      this.cellGraphics.rect(worldX, worldY, this.cellSize, this.cellSize);
      this.cellGraphics.fill({ color: CREATURE_CELL_COLOR, alpha: CREATURE_CELL_ALPHA });
    }

    // Render grid lines
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
