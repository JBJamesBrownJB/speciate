import { Container, Graphics } from 'pixi.js';

const CELL_SIZE = 20;
const HALF_WORLD = 2500;
const CELLS_PER_AXIS = 250;

const CLIFF_COLOR = 0x4a3728;
const CLIFF_BORDER_COLOR = 0x2d2318;

export class TerrainLayer {
  private container: Container;
  private graphics: Graphics;
  private blockedCells = new Set<string>();
  private dirty = true;

  private lastCameraX = 0;
  private lastCameraY = 0;
  private lastZoom = 0;
  private lastViewportWidth = 0;
  private lastViewportHeight = 0;

  constructor(parent: Container) {
    this.container = new Container();
    this.graphics = new Graphics();
    this.container.addChild(this.graphics);
    parent.addChild(this.container);
  }

  setBlockedCells(cells: number[]): void {
    this.blockedCells.clear();

    for (let i = 0; i < cells.length; i += 2) {
      const cellX = cells[i];
      const cellY = cells[i + 1];
      this.blockedCells.add(`${cellX},${cellY}`);
    }

    this.dirty = true;
  }

  setCell(cellX: number, cellY: number, blocked: boolean): void {
    const key = `${cellX},${cellY}`;

    if (blocked) {
      if (!this.blockedCells.has(key)) {
        this.blockedCells.add(key);
        this.dirty = true;
      }
    } else {
      if (this.blockedCells.has(key)) {
        this.blockedCells.delete(key);
        this.dirty = true;
      }
    }
  }

  update(cameraX: number, cameraY: number, zoom: number, viewportWidth: number, viewportHeight: number): void {
    const cameraChanged =
      cameraX !== this.lastCameraX ||
      cameraY !== this.lastCameraY ||
      zoom !== this.lastZoom ||
      viewportWidth !== this.lastViewportWidth ||
      viewportHeight !== this.lastViewportHeight;

    if (!this.dirty && !cameraChanged) return;

    this.lastCameraX = cameraX;
    this.lastCameraY = cameraY;
    this.lastZoom = zoom;
    this.lastViewportWidth = viewportWidth;
    this.lastViewportHeight = viewportHeight;

    this.render(cameraX, cameraY, zoom, viewportWidth, viewportHeight);
    this.dirty = false;
  }

  private render(cameraX: number, cameraY: number, zoom: number, viewportWidth: number, viewportHeight: number): void {
    this.graphics.clear();

    if (this.blockedCells.size === 0) return;

    const halfWidth = viewportWidth / 2 / zoom;
    const halfHeight = viewportHeight / 2 / zoom;

    const minWorldX = cameraX - halfWidth - CELL_SIZE;
    const maxWorldX = cameraX + halfWidth + CELL_SIZE;
    const minWorldY = cameraY - halfHeight - CELL_SIZE;
    const maxWorldY = cameraY + halfHeight + CELL_SIZE;

    const minCellX = Math.max(0, Math.floor((minWorldX + HALF_WORLD) / CELL_SIZE));
    const maxCellX = Math.min(CELLS_PER_AXIS - 1, Math.floor((maxWorldX + HALF_WORLD) / CELL_SIZE));
    const minCellY = Math.max(0, Math.floor((minWorldY + HALF_WORLD) / CELL_SIZE));
    const maxCellY = Math.min(CELLS_PER_AXIS - 1, Math.floor((maxWorldY + HALF_WORLD) / CELL_SIZE));

    for (let cy = minCellY; cy <= maxCellY; cy++) {
      for (let cx = minCellX; cx <= maxCellX; cx++) {
        const key = `${cx},${cy}`;
        if (!this.blockedCells.has(key)) continue;

        const worldX = cx * CELL_SIZE - HALF_WORLD;
        const worldY = cy * CELL_SIZE - HALF_WORLD;

        this.graphics.rect(worldX, worldY, CELL_SIZE, CELL_SIZE);
        this.graphics.fill({ color: CLIFF_COLOR });

        this.graphics.rect(worldX, worldY, CELL_SIZE, CELL_SIZE);
        this.graphics.stroke({ color: CLIFF_BORDER_COLOR, width: 1 });
      }
    }
  }

  getContainer(): Container {
    return this.container;
  }

  destroy(): void {
    this.graphics.destroy();
    this.container.destroy();
  }
}
