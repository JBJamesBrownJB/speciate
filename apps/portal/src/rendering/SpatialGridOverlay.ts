import { Graphics, Container } from 'pixi.js';
import type { QueriedCell } from '@/types/GameState';

const GRID_LINE_COLOR = 0x444444;
const GRID_LINE_ALPHA = 0.6;
const CHECKED_CELL_COLOR = 0x22aa22;
const CHECKED_CELL_ALPHA = 0.35;
const SKIPPED_CELL_COLOR = 0xaa4422;
const SKIPPED_CELL_ALPHA = 0.25;
const CREATURE_CELL_COLOR = 0xdddd22;
const CREATURE_CELL_ALPHA = 0.35;

export class SpatialGridOverlay {
  private graphics: Graphics;
  private cellGraphics: Graphics;
  private visible: boolean = false;
  private cellSize: number = 50;
  private gridMinX: number = -5000;
  private gridMaxX: number = 5000;
  private gridMinY: number = -5000;
  private gridMaxY: number = 5000;
  private checkedCells: QueriedCell[] = [];
  private skippedCells: QueriedCell[] = [];
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

  setBounds(minX: number, maxX: number, minY: number, maxY: number): void {
    // Validate bounds are finite and in correct order
    if (!isFinite(minX) || !isFinite(maxX) || !isFinite(minY) || !isFinite(maxY)) {
      return;
    }
    if (minX >= maxX || minY >= maxY) {
      return;
    }
    this.gridMinX = minX;
    this.gridMaxX = maxX;
    this.gridMinY = minY;
    this.gridMaxY = maxY;
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

  updateQueriedCells(
    queriedCells: QueriedCell[],
    skippedCells: QueriedCell[],
    creatureCell: QueriedCell
  ): void {
    // queriedCells = cells we actually checked (from REAL perception execution)
    // skippedCells = cells we skipped due to early break (from REAL perception execution)
    this.checkedCells = queriedCells;   // Green: cells we checked
    this.skippedCells = skippedCells;   // Orange: cells we skipped
    this.creatureCell = creatureCell;
  }

  clearQueriedCells(): void {
    this.checkedCells = [];
    this.skippedCells = [];
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

    // Viewport bounds in world coordinates
    const viewLeft = cameraX - halfViewW;
    const viewRight = cameraX + halfViewW;
    const viewTop = cameraY - halfViewH;
    const viewBottom = cameraY + halfViewH;

    // Clamp to actual spatial grid bounds
    const worldLeft = Math.max(viewLeft, this.gridMinX);
    const worldRight = Math.min(viewRight, this.gridMaxX);
    const worldTop = Math.max(viewTop, this.gridMinY);
    const worldBottom = Math.min(viewBottom, this.gridMaxY);

    // If grid is completely outside viewport, nothing to render
    if (worldLeft >= worldRight || worldTop >= worldBottom) {
      return;
    }

    // Guard against invalid cell size
    if (this.cellSize <= 0 || !isFinite(this.cellSize)) {
      return;
    }

    const startCellX = Math.floor(worldLeft / this.cellSize);
    const endCellX = Math.ceil(worldRight / this.cellSize);
    const startCellY = Math.floor(worldTop / this.cellSize);
    const endCellY = Math.ceil(worldBottom / this.cellSize);

    // Guard against too many cells (prevent infinite loops)
    const maxCells = 10000;
    if ((endCellX - startCellX) * (endCellY - startCellY) > maxCells) {
      return;
    }

    // Render skipped cells (orange/red fill) - cells in range but skipped due to early break
    if (this.skippedCells.length > 0) {
      for (const cell of this.skippedCells) {
        const worldX = cell.x * this.cellSize;
        const worldY = cell.y * this.cellSize;
        this.cellGraphics.rect(worldX, worldY, this.cellSize, this.cellSize);
      }
      this.cellGraphics.fill({ color: SKIPPED_CELL_COLOR, alpha: SKIPPED_CELL_ALPHA });
    }

    // Render checked cells (green fill) - cells actually examined
    if (this.checkedCells.length > 0) {
      for (const cell of this.checkedCells) {
        const worldX = cell.x * this.cellSize;
        const worldY = cell.y * this.cellSize;
        this.cellGraphics.rect(worldX, worldY, this.cellSize, this.cellSize);
      }
      this.cellGraphics.fill({ color: CHECKED_CELL_COLOR, alpha: CHECKED_CELL_ALPHA });
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
