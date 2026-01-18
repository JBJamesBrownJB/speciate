import { Graphics, Container } from 'pixi.js';
import type { QueriedCell, L1VisionDebugEntry } from '@/types/GameState';
import type { IOverlay, OverlayConfig } from './IOverlay';

const L0_GRID_LINE_COLOR = 0x444444;
const L1_GRID_LINE_COLOR = 0x00aaff;
const GRID_LINE_ALPHA = 0.6;
const CHECKED_CELL_COLOR = 0x22aa22;
const CHECKED_CELL_ALPHA = 0.35;
const SKIPPED_CELL_COLOR = 0xaa4422;
const SKIPPED_CELL_ALPHA = 0.25;
const CREATURE_CELL_COLOR = 0xdddd22;
const CREATURE_CELL_ALPHA = 0.35;
const L1_VISION_CELL_COLOR = 0x888888;
const L1_VISION_CELL_ALPHA = 0.35;

interface L1CellInfo {
  cellX: number;
  cellY: number;
  worldCenterX: number;
  worldCenterY: number;
  creatureCount: number;
  totalMass: number;
  maxSize: number;
  avgSize: number;
}

export enum GridMode {
  Off = 'off',
  L0 = 'l0',
  L1 = 'l1',
}

const MODE_ORDER: GridMode[] = [
  GridMode.Off,
  GridMode.L0,
  GridMode.L1,
];

export class SpatialGridOverlay implements IOverlay {
  readonly config: OverlayConfig = {
    name: 'spatialGrid',
    devToolsOnly: true,
    keyboardShortcut: 'g',
  };

  private graphics: Graphics;
  private cellGraphics: Graphics;
  private currentMode: GridMode = GridMode.Off;
  private l0CellSize: number = 10;
  private l1CellSize: number = 30;
  private gridMinX: number = -5000;
  private gridMaxX: number = 5000;
  private gridMinY: number = -5000;
  private gridMaxY: number = 5000;
  private checkedCells: QueriedCell[] = [];
  private skippedCells: QueriedCell[] = [];
  private creatureCell: QueriedCell | null = null;

  // L1 vision cells (creatures' perceived L1 cells)
  private l1VisionCells: L1VisionDebugEntry[] = [];

  // Info panel for L1 hover
  private infoPanel: HTMLDivElement | null = null;

  // L1 hover state (managed by InteractionManager)
  private hoveredInfo: L1CellInfo | null = null;
  private hoverPending: boolean = false;

  constructor(container: Container) {
    this.cellGraphics = new Graphics();
    this.cellGraphics.visible = false;
    container.addChild(this.cellGraphics);

    this.graphics = new Graphics();
    this.graphics.visible = false;
    container.addChild(this.graphics);

    this.createInfoPanel();
  }

  private createInfoPanel(): void {
    this.infoPanel = document.createElement('div');
    this.infoPanel.id = 'l1-cell-info-panel';
    this.infoPanel.style.cssText = `
      position: fixed;
      top: 10px;
      right: 250px;
      background: rgba(0, 0, 0, 0.85);
      color: #00ffff;
      font-family: 'Consolas', 'Monaco', monospace;
      font-size: 12px;
      padding: 12px 16px;
      border-radius: 6px;
      border: 1px solid #00aaff;
      z-index: 1000;
      display: none;
      min-width: 200px;
      box-shadow: 0 4px 12px rgba(0, 170, 255, 0.3);
    `;
    document.body.appendChild(this.infoPanel);
  }

  handleHover(worldX: number, worldY: number): void {
    if (this.currentMode !== GridMode.L1) return;
    if (this.hoverPending) return;

    this.hoverPending = true;

    requestAnimationFrame(async () => {
      try {
        const info = await window.electron?.queryL1Cell?.(worldX, worldY);
        if (info) {
          this.hoveredInfo = info;
        } else {
          // Backend returns null for empty cells - create minimal info
          const cellX = Math.floor(worldX / this.l1CellSize);
          const cellY = Math.floor(worldY / this.l1CellSize);
          this.hoveredInfo = {
            cellX,
            cellY,
            worldCenterX: (cellX + 0.5) * this.l1CellSize,
            worldCenterY: (cellY + 0.5) * this.l1CellSize,
            creatureCount: 0,
            totalMass: 0,
            maxSize: 0,
            avgSize: 0,
          };
        }
        this.updateInfoPanel();
      } finally {
        this.hoverPending = false;
      }
    });
  }

  clearHover(): void {
    this.hoveredInfo = null;
    this.hoverPending = false;
    if (this.infoPanel) {
      this.infoPanel.style.display = 'none';
    }
  }

  private updateInfoPanel(): void {
    if (!this.infoPanel) return;

    if (!this.hoveredInfo || this.currentMode !== GridMode.L1) {
      this.infoPanel.style.display = 'none';
      return;
    }

    const info = this.hoveredInfo;
    this.infoPanel.innerHTML = `
      <div style="font-weight: bold; margin-bottom: 8px; color: #00aaff; border-bottom: 1px solid #00aaff40; padding-bottom: 6px;">
        L1 Cell (${info.cellX}, ${info.cellY})
      </div>
      <div style="display: grid; grid-template-columns: auto auto; gap: 4px 12px;">
        <span style="color: #888;">World Center:</span>
        <span style="text-align: right;">(${info.worldCenterX.toFixed(1)}, ${info.worldCenterY.toFixed(1)})</span>
        <span style="color: #888;">Creatures:</span>
        <span style="text-align: right;">${info.creatureCount}</span>
        <span style="color: #888;">Total Mass:</span>
        <span style="text-align: right;">${info.totalMass.toFixed(1)} kg</span>
        <span style="color: #888;">Avg Size:</span>
        <span style="text-align: right;">${info.avgSize.toFixed(2)} m</span>
        <span style="color: #888;">Max Size:</span>
        <span style="text-align: right;">${info.maxSize.toFixed(2)} m</span>
      </div>
    `;
    this.infoPanel.style.display = 'block';
  }

  setCellSize(cellSize: number): void {
    this.l0CellSize = cellSize;
  }

  setL1CellSize(cellSize: number): void {
    this.l1CellSize = cellSize;
  }

  setBounds(minX: number, maxX: number, minY: number, maxY: number): void {
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

  getMode(): GridMode {
    return this.currentMode;
  }

  toggle(): void {
    const currentIndex = MODE_ORDER.indexOf(this.currentMode);
    const nextIndex = (currentIndex + 1) % MODE_ORDER.length;
    this.setMode(MODE_ORDER[nextIndex]);
  }

  private setMode(mode: GridMode): void {
    this.currentMode = mode;
    const isVisible = mode !== GridMode.Off;
    this.graphics.visible = isVisible;
    this.cellGraphics.visible = isVisible;

    // Clear hover state when leaving L1 mode
    if (mode !== GridMode.L1) {
      this.clearHover();
    }
  }

  show(): void {
    if (this.currentMode === GridMode.Off) {
      this.setMode(GridMode.L0);
    }
  }

  hide(): void {
    this.setMode(GridMode.Off);
  }

  updateQueriedCells(
    queriedCells: QueriedCell[],
    skippedCells: QueriedCell[],
    creatureCell: QueriedCell
  ): void {
    this.checkedCells = queriedCells;
    this.skippedCells = skippedCells;
    this.creatureCell = creatureCell;
  }

  clearQueriedCells(): void {
    this.checkedCells = [];
    this.skippedCells = [];
    this.creatureCell = null;
  }

  updateL1VisionCells(l1Vision: L1VisionDebugEntry[] | undefined): void {
    this.l1VisionCells = l1Vision ?? [];
  }

  clearL1VisionCells(): void {
    this.l1VisionCells = [];
  }

  isVisible(): boolean {
    return this.currentMode !== GridMode.Off;
  }

  update(
    cameraX: number,
    cameraY: number,
    zoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): void {
    if (this.currentMode === GridMode.Off) return;

    switch (this.currentMode) {
      case GridMode.L0:
        this.renderL0Grid(cameraX, cameraY, zoom, viewportWidth, viewportHeight);
        break;
      case GridMode.L1:
        this.renderL1Grid(cameraX, cameraY, zoom, viewportWidth, viewportHeight);
        break;
    }
  }

  destroy(): void {
    // Remove info panel
    if (this.infoPanel && this.infoPanel.parentNode) {
      this.infoPanel.parentNode.removeChild(this.infoPanel);
    }

    this.graphics.destroy();
    this.cellGraphics.destroy();
  }

  private renderL0Grid(
    cameraX: number,
    cameraY: number,
    zoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): void {
    this.graphics.clear();
    this.cellGraphics.clear();

    const cellSize = this.l0CellSize;
    const { worldLeft, worldRight, worldTop, worldBottom } = this.getViewBounds(
      cameraX, cameraY, zoom, viewportWidth, viewportHeight
    );

    if (worldLeft >= worldRight || worldTop >= worldBottom) return;
    if (cellSize <= 0 || !isFinite(cellSize)) return;

    const startCellX = Math.floor(worldLeft / cellSize);
    const endCellX = Math.ceil(worldRight / cellSize);
    const startCellY = Math.floor(worldTop / cellSize);
    const endCellY = Math.ceil(worldBottom / cellSize);

    const maxCells = 10000;
    if ((endCellX - startCellX) * (endCellY - startCellY) > maxCells) return;

    if (this.skippedCells.length > 0) {
      for (const cell of this.skippedCells) {
        const worldX = cell.x * cellSize;
        const worldY = cell.y * cellSize;
        this.cellGraphics.rect(worldX, worldY, cellSize, cellSize);
      }
      this.cellGraphics.fill({ color: SKIPPED_CELL_COLOR, alpha: SKIPPED_CELL_ALPHA });
    }

    if (this.checkedCells.length > 0) {
      for (const cell of this.checkedCells) {
        const worldX = cell.x * cellSize;
        const worldY = cell.y * cellSize;
        this.cellGraphics.rect(worldX, worldY, cellSize, cellSize);
      }
      this.cellGraphics.fill({ color: CHECKED_CELL_COLOR, alpha: CHECKED_CELL_ALPHA });
    }

    if (this.creatureCell) {
      const worldX = this.creatureCell.x * cellSize;
      const worldY = this.creatureCell.y * cellSize;
      this.cellGraphics.rect(worldX, worldY, cellSize, cellSize);
      this.cellGraphics.fill({ color: CREATURE_CELL_COLOR, alpha: CREATURE_CELL_ALPHA });
    }

    this.renderGridLines(cellSize, worldLeft, worldRight, worldTop, worldBottom, zoom, L0_GRID_LINE_COLOR);
  }

  private renderL1Grid(
    cameraX: number,
    cameraY: number,
    zoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): void {
    this.graphics.clear();
    this.cellGraphics.clear();

    const cellSize = this.l1CellSize;
    const { worldLeft, worldRight, worldTop, worldBottom } = this.getViewBounds(
      cameraX, cameraY, zoom, viewportWidth, viewportHeight
    );

    if (worldLeft >= worldRight || worldTop >= worldBottom) return;
    if (cellSize <= 0 || !isFinite(cellSize)) return;

    // Draw L1 vision cells (gray highlight showing perceived L1 cells)
    if (this.l1VisionCells.length > 0) {
      for (const entry of this.l1VisionCells) {
        // Calculate cell world position from center coordinates
        const cellX = Math.floor(entry.centerX / cellSize);
        const cellY = Math.floor(entry.centerY / cellSize);
        const worldX = cellX * cellSize;
        const worldY = cellY * cellSize;
        this.cellGraphics.rect(worldX, worldY, cellSize, cellSize);
      }
      this.cellGraphics.fill({ color: L1_VISION_CELL_COLOR, alpha: L1_VISION_CELL_ALPHA });
    }

    this.renderGridLines(cellSize, worldLeft, worldRight, worldTop, worldBottom, zoom, L1_GRID_LINE_COLOR);
  }

  private getViewBounds(
    cameraX: number,
    cameraY: number,
    zoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): { worldLeft: number; worldRight: number; worldTop: number; worldBottom: number } {
    const halfViewW = viewportWidth / 2 / zoom;
    const halfViewH = viewportHeight / 2 / zoom;

    const viewLeft = cameraX - halfViewW;
    const viewRight = cameraX + halfViewW;
    const viewTop = cameraY - halfViewH;
    const viewBottom = cameraY + halfViewH;

    return {
      worldLeft: Math.max(viewLeft, this.gridMinX),
      worldRight: Math.min(viewRight, this.gridMaxX),
      worldTop: Math.max(viewTop, this.gridMinY),
      worldBottom: Math.min(viewBottom, this.gridMaxY),
    };
  }

  private renderGridLines(
    cellSize: number,
    worldLeft: number,
    worldRight: number,
    worldTop: number,
    worldBottom: number,
    zoom: number,
    color: number
  ): void {
    const startCellX = Math.floor(worldLeft / cellSize);
    const endCellX = Math.ceil(worldRight / cellSize);
    const startCellY = Math.floor(worldTop / cellSize);
    const endCellY = Math.ceil(worldBottom / cellSize);

    const maxCells = 10000;
    if ((endCellX - startCellX) * (endCellY - startCellY) > maxCells) return;

    const lineWidth = 1.0 / zoom;

    this.graphics.setStrokeStyle({
      width: lineWidth,
      color,
      alpha: GRID_LINE_ALPHA,
    });

    for (let cellX = startCellX; cellX <= endCellX; cellX++) {
      const x = cellX * cellSize;
      this.graphics.moveTo(x, worldTop);
      this.graphics.lineTo(x, worldBottom);
    }

    for (let cellY = startCellY; cellY <= endCellY; cellY++) {
      const y = cellY * cellSize;
      this.graphics.moveTo(worldLeft, y);
      this.graphics.lineTo(worldRight, y);
    }

    this.graphics.stroke();
  }
}
